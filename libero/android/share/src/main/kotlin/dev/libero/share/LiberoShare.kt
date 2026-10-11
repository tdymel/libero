package dev.libero.share

import android.app.Activity
import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.database.MatrixCursor
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import android.webkit.MimeTypeMap
import java.io.File
import java.io.FileNotFoundException
import java.util.UUID

/** Called from Rust over manganis `#[ffi]`: stages a file to share and names its URI. */
class LiberoShare(private val activity: Activity) {
    /** A fresh path for a file called `name`; shares older than a day are removed first. */
    fun stagePath(name: String): String {
        val root = LiberoShareProvider.root(activity)
        val old = System.currentTimeMillis() - 24 * 60 * 60 * 1000L
        root.listFiles()?.filter { it.lastModified() < old }?.forEach { it.deleteRecursively() }
        val folder = File(root, UUID.randomUUID().toString())
        folder.mkdirs()
        val safe = name.replace('/', '_').replace('\u0000', '_').ifBlank { "file" }
        return File(folder, safe).path
    }

    fun contentUri(path: String): String {
        val file = File(path)
        return Uri.Builder()
            .scheme("content")
            .authority(LiberoShareProvider.authority(activity))
            .appendPath(file.parentFile?.name ?: "")
            .appendPath(file.name)
            .build()
            .toString()
    }
}

/**
 * Read-only files of [LiberoShare]. It answers a file's name and size, and null for any other
 * column, so a chooser that asks for document columns still gets the name (todo 3408).
 */
class LiberoShareProvider : ContentProvider() {
    companion object {
        fun root(context: Context) = File(context.cacheDir, "libero-share")

        fun authority(context: Context) = "${context.packageName}.libero.share"
    }

    override fun onCreate() = true

    private fun file(uri: Uri): File {
        val context = context ?: throw FileNotFoundException("no context")
        val segments = uri.pathSegments
        if (segments.size != 2) throw FileNotFoundException(uri.toString())
        val root = root(context).canonicalFile
        val file = File(File(root, segments[0]), segments[1]).canonicalFile
        if (file.parentFile?.parentFile != root || !file.isFile) {
            throw FileNotFoundException(uri.toString())
        }
        return file
    }

    override fun query(
        uri: Uri,
        projection: Array<String>?,
        selection: String?,
        selectionArgs: Array<String>?,
        sortOrder: String?,
    ): Cursor {
        val file = file(uri)
        val columns = projection ?: arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE)
        val row = columns.map {
            when (it) {
                OpenableColumns.DISPLAY_NAME -> file.name
                OpenableColumns.SIZE -> file.length()
                else -> null
            }
        }
        return MatrixCursor(columns, 1).apply { addRow(row) }
    }

    override fun getType(uri: Uri): String {
        val extension = uri.lastPathSegment?.substringAfterLast('.', "")?.lowercase() ?: ""
        return MimeTypeMap.getSingleton().getMimeTypeFromExtension(extension)
            ?: "application/octet-stream"
    }

    override fun openFile(uri: Uri, mode: String): ParcelFileDescriptor {
        if (mode != "r") throw SecurityException("read only")
        return ParcelFileDescriptor.open(file(uri), ParcelFileDescriptor.MODE_READ_ONLY)
    }

    override fun insert(uri: Uri, values: ContentValues?): Uri? = null

    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<String>?,
    ) = 0

    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<String>?) = 0
}
