use dioxus::prelude::*;

/// The glyphs components cannot do without; every new one goes here (todo 95). Sizeless,
/// `currentColor` and `aria-hidden`: the caller sizes the box around it.
#[component]
pub(crate) fn CloseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 6L6 18" }
            path { d: "M6 6l12 12" }
        }
    }
}

/// Points down while its disclosure is closed; the caller rotates it.
#[component]
pub(crate) fn ChevronDownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M6 9l6 6 6-6" }
        }
    }
}

/// Two bars, filled so they keep their weight at small sizes.
#[component]
pub(crate) fn PauseIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path { d: "M6 5h4v14H6zM14 5h4v14h-4z" }
        }
    }
}

/// A right-pointing triangle, [`PauseIcon`]'s other state.
#[component]
pub(crate) fn PlayIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path { d: "M8 5v14l11-7z" }
        }
    }
}

/// A done mark: a completed step's marker.
#[component]
pub(crate) fn CheckIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M5 12l5 5 9-10" }
        }
    }
}

/// Points at a submenu; stays pointing right even when the submenu flips, as native menus do.
#[component]
pub(crate) fn ChevronRightIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M9 6l6 6-6 6" }
        }
    }
}

/// Points back: a calendar's previous month, a pager's previous page.
#[component]
pub(crate) fn ChevronLeftIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 6l-6 6 6 6" }
        }
    }
}

/// Points up: a vertical carousel's previous slide.
#[component]
pub(crate) fn ChevronUpIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M18 15l-6-6-6 6" }
        }
    }
}

/// A chevron against a bar, so "first" does not read as one more step back.
#[component]
pub(crate) fn ChevronFirstIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M17 6l-6 6 6 6" }
            path { d: "M7 6v12" }
        }
    }
}

/// [`ChevronFirstIcon`] mirrored.
#[component]
pub(crate) fn ChevronLastIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M7 6l6 6-6 6" }
            path { d: "M17 6v12" }
        }
    }
}

/// A sortable column's arrow. The header flips it for ascending.
#[component]
pub(crate) fn ArrowDownIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 5v14" }
            path { d: "M6 13l6 6 6-6" }
        }
    }
}

/// A number field's step down.
#[component]
pub(crate) fn MinusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M6 12h12" }
        }
    }
}

/// A number field's step up.
#[component]
pub(crate) fn PlusIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            "aria-hidden": "true",
            path { d: "M12 6v12" }
            path { d: "M6 12h12" }
        }
    }
}

/// A password field's reveal button, while the secret is hidden.
#[component]
pub(crate) fn EyeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z" }
            circle { cx: "12", cy: "12", r: "3" }
        }
    }
}

/// [`EyeIcon`] struck through, while the secret is shown.
#[component]
pub(crate) fn EyeOffIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M2 12s3.5-7 10-7c2 0 3.8.7 5.2 1.6" }
            path { d: "M21.5 10.4c.3.6.5 1.1.5 1.6 0 0-3.5 7-10 7-1.3 0-2.5-.3-3.5-.7" }
            path { d: "M9.9 9.9a3 3 0 0 0 4.2 4.2" }
            path { d: "M3 3l18 18" }
        }
    }
}

/// A file field's dropzone prompt: a tray with an arrow going into it.
#[component]
pub(crate) fn UploadIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M12 16V4" }
            path { d: "M8 8l4-4 4 4" }
            path { d: "M4 16v2a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" }
        }
    }
}

/// `ThemeToggle` when a press hands the choice back to the platform: a
/// disc half filled, the usual "automatic" mark.
#[component]
pub(crate) fn SystemSchemeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 3a9 9 0 0 0 0 18z", fill: "currentColor" }
        }
    }
}

/// `ThemeToggle` when a press switches to light.
#[component]
pub(crate) fn SunIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "4" }
            path { d: "M12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M19.1 4.9l-1.4 1.4M6.3 17.7l-1.4 1.4" }
        }
    }
}

/// `ThemeToggle` when a press switches to dark.
#[component]
pub(crate) fn MoonIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M20 14.5A8.5 8.5 0 1 1 9.5 4a6.6 6.6 0 0 0 10.5 10.5z" }
        }
    }
}

/// `RepoButton`'s GitHub mark.
#[component]
pub(crate) fn GitHubIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 16 16",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82a7.4 7.4 0 0 1 2-.27c.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
            }
        }
    }
}

/// `RepoButton`'s GitLab mark (Simple Icons, CC0).
#[component]
pub(crate) fn GitLabIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "m23.6004 9.5927-.0337-.0862L20.3.9814a.851.851 0 0 0-.3362-.405.8748.8748 0 0 0-.9997.0539.8748.8748 0 0 0-.29.4399l-2.2055 6.748H7.5375l-2.2057-6.748a.8573.8573 0 0 0-.29-.4412.8748.8748 0 0 0-.9997-.0537.8585.8585 0 0 0-.3362.4049L.4332 9.5015l-.0325.0862a6.0657 6.0657 0 0 0 2.0119 7.0105l.0113.0087.03.0213 4.976 3.7264 2.462 1.8633 1.4995 1.1321a1.0085 1.0085 0 0 0 1.2197 0l1.4995-1.1321 2.4619-1.8633 5.006-3.7489.0125-.01a6.0682 6.0682 0 0 0 2.0094-7.003z"
            }
        }
    }
}

/// `Tldr`'s trigger: a four-point star with two small ones (Lucide `sparkles`, ISC).
#[component]
pub(crate) fn SparklesIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M9.937 15.5A2 2 0 0 0 8.5 14.063l-6.135-1.582a.5.5 0 0 1 0-.962L8.5 9.936A2 2 0 0 0 9.937 8.5l1.582-6.135a.5.5 0 0 1 .963 0L14.063 8.5A2 2 0 0 0 15.5 9.937l6.135 1.581a.5.5 0 0 1 0 .964L15.5 14.063a2 2 0 0 0-1.437 1.437l-1.582 6.135a.5.5 0 0 1-.963 0z" }
            path { d: "M20 3v4M22 5h-4M4 17v2M5 18H3" }
        }
    }
}

/// `Tldr`'s ChatGPT mark (Lobe Icons `openai`, MIT, Copyright (c) 2023 LobeHub).
/// A trademark of OpenAI, shown only to name the service a link opens.
#[component]
pub(crate) fn ChatGptIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                fill_rule: "evenodd",
                d: "M9.205 8.658v-2.26c0-.19.072-.333.238-.428l4.543-2.616c.619-.357 1.356-.523 2.117-.523 2.854 0 4.662 2.212 4.662 4.566 0 .167 0 .357-.024.547l-4.71-2.759a.797.797 0 00-.856 0l-5.97 3.473zm10.609 8.8V12.06c0-.333-.143-.57-.429-.737l-5.97-3.473 1.95-1.118a.433.433 0 01.476 0l4.543 2.617c1.309.76 2.189 2.378 2.189 3.948 0 1.808-1.07 3.473-2.76 4.163zM7.802 12.703l-1.95-1.142c-.167-.095-.239-.238-.239-.428V5.899c0-2.545 1.95-4.472 4.591-4.472 1 0 1.927.333 2.712.928L8.23 5.067c-.285.166-.428.404-.428.737v6.898zM12 15.128l-2.795-1.57v-3.33L12 8.658l2.795 1.57v3.33L12 15.128zm1.796 7.23c-1 0-1.927-.332-2.712-.927l4.686-2.712c.285-.166.428-.404.428-.737v-6.898l1.974 1.142c.167.095.238.238.238.428v5.233c0 2.545-1.974 4.472-4.614 4.472zm-5.637-5.303l-4.544-2.617c-1.308-.761-2.188-2.378-2.188-3.948A4.482 4.482 0 014.21 6.327v5.423c0 .333.143.571.428.738l5.947 3.449-1.95 1.118a.432.432 0 01-.476 0zm-.262 3.9c-2.688 0-4.662-2.021-4.662-4.519 0-.19.024-.38.047-.57l4.686 2.71c.286.167.571.167.856 0l5.97-3.448v2.26c0 .19-.07.333-.237.428l-4.543 2.616c-.619.357-1.356.523-2.117.523zm5.899 2.83a5.947 5.947 0 005.827-4.756C22.287 18.339 24 15.84 24 13.296c0-1.665-.713-3.282-1.998-4.448.119-.5.19-.999.19-1.498 0-3.401-2.759-5.947-5.946-5.947-.642 0-1.26.095-1.88.31A5.962 5.962 0 0010.205 0a5.947 5.947 0 00-5.827 4.757C1.713 5.447 0 7.945 0 10.49c0 1.666.713 3.283 1.998 4.448-.119.5-.19 1-.19 1.499 0 3.401 2.759 5.946 5.946 5.946.642 0 1.26-.095 1.88-.309a5.96 5.96 0 004.162 1.713z",
            }
        }
    }
}

/// `Tldr`'s Google mark (Simple Icons `google`, CC0). A trademark of Google.
#[component]
pub(crate) fn GoogleIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M12.48 10.92v3.28h7.84c-.24 1.84-.853 3.187-1.787 4.133-1.147 1.147-2.933 2.4-6.053 2.4-4.827 0-8.6-3.893-8.6-8.72s3.773-8.72 8.6-8.72c2.6 0 4.507 1.027 5.907 2.347l2.307-2.307C18.747 1.44 16.133 0 12.48 0 5.867 0 .307 5.387.307 12s5.56 12 12.173 12c3.573 0 6.267-1.173 8.373-3.36 2.16-2.16 2.84-5.213 2.84-7.667 0-.76-.053-1.467-.173-2.053H12.48z",
            }
        }
    }
}

/// `Tldr`'s Claude mark (Simple Icons `claude`, CC0). A trademark of Anthropic.
#[component]
pub(crate) fn ClaudeIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "m4.7144 15.9555 4.7174-2.6471.079-.2307-.079-.1275h-.2307l-.7893-.0486-2.6956-.0729-2.3375-.0971-2.2646-.1214-.5707-.1215-.5343-.7042.0546-.3522.4797-.3218.686.0608 1.5179.1032 2.2767.1578 1.6514.0972 2.4468.255h.3886l.0546-.1579-.1336-.0971-.1032-.0972L6.973 9.8356l-2.55-1.6879-1.3356-.9714-.7225-.4918-.3643-.4614-.1578-1.0078.6557-.7225.8803.0607.2246.0607.8925.686 1.9064 1.4754 2.4893 1.8336.3643.3035.1457-.1032.0182-.0728-.164-.2733-1.3539-2.4467-1.445-2.4893-.6435-1.032-.17-.6194c-.0607-.255-.1032-.4674-.1032-.7285L6.287.1335 6.6997 0l.9957.1336.419.3642.6192 1.4147 1.0018 2.2282 1.5543 3.0296.4553.8985.2429.8318.091.255h.1579v-.1457l.1275-1.706.2368-2.0947.2307-2.6957.0789-.7589.3764-.9107.7468-.4918.5828.2793.4797.686-.0668.4433-.2853 1.8517-.5586 2.9021-.3643 1.9429h.2125l.2429-.2429.9835-1.3053 1.6514-2.0643.7286-.8196.85-.9046.5464-.4311h1.0321l.759 1.1293-.34 1.1657-1.0625 1.3478-.8804 1.1414-1.2628 1.7-.7893 1.36.0729.1093.1882-.0183 2.8535-.607 1.5421-.2794 1.8396-.3157.8318.3886.091.3946-.3278.8075-1.967.4857-2.3072.4614-3.4364.8136-.0425.0304.0486.0607 1.5482.1457.6618.0364h1.621l3.0175.2247.7892.522.4736.6376-.079.4857-1.2142.6193-1.6393-.3886-3.825-.9107-1.3113-.3279h-.1822v.1093l1.0929 1.0686 2.0035 1.8092 2.5075 2.3314.1275.5768-.3218.4554-.34-.0486-2.2039-1.6575-.85-.7468-1.9246-1.621h-.1275v.17l.4432.6496 2.3436 3.5214.1214 1.0807-.17.3521-.6071.2125-.6679-.1214-1.3721-1.9246L14.38 17.959l-1.1414-1.9428-.1397.079-.674 7.2552-.3156.3703-.7286.2793-.6071-.4614-.3218-.7468.3218-1.4753.3886-1.9246.3157-1.53.2853-1.9004.17-.6314-.0121-.0425-.1397.0182-1.4328 1.9672-2.1796 2.9446-1.7243 1.8456-.4128.164-.7164-.3704.0667-.6618.4008-.5889 2.386-3.0357 1.4389-1.882.929-1.0868-.0062-.1579h-.0546l-6.3385 4.1164-1.1293.1457-.4857-.4554.0608-.7467.2307-.2429 1.9064-1.3114Z",
            }
        }
    }
}

/// `Tldr`'s Perplexity mark (Simple Icons `perplexity`, CC0). A trademark of Perplexity.
#[component]
pub(crate) fn PerplexityIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M22.3977 7.0896h-2.3106V.0676l-7.5094 6.3542V.1577h-1.1554v6.1966L4.4904 0v7.0896H1.6023v10.3976h2.8882V24l6.932-6.3591v6.2005h1.1554v-6.0469l6.9318 6.1807v-6.4879h2.8882V7.0896zm-3.4657-4.531v4.531h-5.355l5.355-4.531zm-13.2862.0676 4.8691 4.4634H5.6458V2.6262zM2.7576 16.332V8.245h7.8476l-6.1149 6.1147v1.9723H2.7576zm2.8882 5.0404v-3.8852h.0001v-2.6488l5.7763-5.7764v7.0111l-5.7764 5.2993zm12.7086.0248-5.7766-5.1509V9.0618l5.7766 5.7766v6.5588zm2.8882-5.0652h-1.733v-1.9723L13.3948 8.245h7.8478v8.087z",
            }
        }
    }
}

/// `DirectionToggle`: a pilcrow over an arrow pointing where a press turns
/// the text, left for right to left.
#[component]
pub(crate) fn TextDirectionIcon(to_rtl: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 3v11" }
            path { d: "M15 3v11" }
            path { d: "M18 3h-7.5a4 4 0 0 0 0 8h.5" }
            path { d: "M20 19H4" }
            path { d: if to_rtl { "M7 16l-3 3 3 3" } else { "M17 16l3 3-3 3" } }
        }
    }
}

/// `ColorField`'s eyedropper button: a pipette, tip at the bottom left.
#[component]
pub(crate) fn EyeDropperIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M11 7l6 6" }
            path { d: "M4 16l11.7 -11.7a1 1 0 0 1 1.4 0l2.6 2.6a1 1 0 0 1 0 1.4l-11.7 11.7h-4v-4z" }
        }
    }
}

/// A code block's copy button.
#[component]
pub(crate) fn CopyIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            rect { x: "9", y: "9", width: "13", height: "13", rx: "2" }
            path { d: "M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1" }
        }
    }
}

/// [`CopyIcon`]'s other state. Not [`CheckIcon`]'s path, so nothing visible changed.
#[component]
pub(crate) fn CopiedIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            polyline { points: "20 6 9 17 4 12" }
        }
    }
}

/// [`CopyIcon`]'s state after a denied write: a circled exclamation mark.
#[component]
pub(crate) fn CopyFailedIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            circle { cx: "12", cy: "12", r: "9" }
            path { d: "M12 8v4" }
            path { d: "M12 16h.01" }
        }
    }
}

/// An `Anchor` that opens a new tab: a box with an arrow leaving it.
#[component]
pub(crate) fn ExternalLinkIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            stroke_width: "2",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            path { d: "M15 3h6v6" }
            path { d: "M10 14L21 3" }
            path { d: "M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6" }
        }
    }
}

/// The last link in `Avatar`'s fallback chain: a head and shoulders.
#[component]
pub(crate) fn PersonIcon() -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "currentColor",
            "aria-hidden": "true",
            path {
                d: "M12 12a5 5 0 1 0 0-10 5 5 0 0 0 0 10Zm0 1.8c-4.1 0-7.4 2.1-7.4 4.7V21h14.8v-2.5c0-2.6-3.3-4.7-7.4-4.7Z",
            }
        }
    }
}

/// A checkbox's mark: the check, or the dash while `indeterminate`. Heavier
/// than the other glyphs (stroke 3), since it is drawn at 65% of a small box.
#[component]
pub(crate) fn CheckboxMarkIcon(indeterminate: bool) -> Element {
    rsx! {
        svg {
            view_box: "0 0 24 24",
            fill: "none",
            // This casing: Blitz's svg parser matches `currentColor` exactly.
            stroke: "currentColor",
            stroke_width: "3",
            stroke_linecap: "round",
            stroke_linejoin: "round",
            "aria-hidden": "true",
            if indeterminate {
                path { d: "M6 12h12" }
            } else {
                path { d: "M5 13l4 4L19 7" }
            }
        }
    }
}
