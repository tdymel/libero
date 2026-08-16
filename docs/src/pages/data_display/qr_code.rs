use dioxus::prelude::*;
use libero::{
    components::{Code, Flex, QrCode, Text, Title},
    sx::sx,
};

#[component]
pub fn QrCodePage() -> Element {
    rsx! {
        Flex {
            direction: "column",
            gap: "32px",
            Flex {
                direction: "column",
                gap: "16px",
                Title { variant: "h1", "QrCode" }
                Text {
                    "Encodes "
                    Code { "data" }
                    " as a scalable QR code, rendered as an inline SVG. Background/"
                    "foreground colors come from the theme ("
                    Code { "Theme::qr_code" }
                    "), not per-instance props - "
                    Code { "robustness" }
                    " is the only thing you tune per code."
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Basic usage" }
                QrCode {
                    data: "https://github.com/tdymel/libero",
                    aria_label: "QR code linking to the libero GitHub repository",
                    sx: sx().width("160px"),
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Robustness" }
                Text {
                    sx: sx().color("grey.6"),
                    "Error-correction level - higher levels tolerate more damage/"
                    "occlusion at the cost of a denser code for the same data. Falls "
                    "back to the theme's "
                    Code { "Theme::qr_code.robustness" }
                    " (Medium) when unset."
                }
                Flex {
                    direction: "row",
                    gap: "24px",
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        QrCode {
                            data: "https://github.com/tdymel/libero",
                            robustness: "low",
                            aria_label: "QR code, low error correction",
                            sx: sx().width("120px"),
                        }
                        Text { size: "sm", "Low" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        QrCode {
                            data: "https://github.com/tdymel/libero",
                            robustness: "medium",
                            aria_label: "QR code, medium error correction",
                            sx: sx().width("120px"),
                        }
                        Text { size: "sm", "Medium" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        QrCode {
                            data: "https://github.com/tdymel/libero",
                            robustness: "quartile",
                            aria_label: "QR code, quartile error correction",
                            sx: sx().width("120px"),
                        }
                        Text { size: "sm", "Quartile" }
                    }
                    Flex {
                        direction: "column",
                        align: "center",
                        gap: "8px",
                        QrCode {
                            data: "https://github.com/tdymel/libero",
                            robustness: "high",
                            aria_label: "QR code, high error correction",
                            sx: sx().width("120px"),
                        }
                        Text { size: "sm", "High" }
                    }
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Scalable" }
                Text {
                    sx: sx().color("grey.6"),
                    "The generated SVG has no fixed width/height, just a square "
                    Code { "viewBox" }
                    " - it fills its container, so "
                    Code { "sx" }
                    " width/height (or the container's own size) controls the "
                    "rendered size."
                }
                QrCode {
                    data: "Scaled via a 240px wrapper",
                    aria_label: "QR code demonstrating scaling",
                    sx: sx().width("240px"),
                }
            }

            Flex {
                direction: "column",
                gap: "8px",
                Title { variant: "h2", "Accessible name" }
                Text {
                    sx: sx().color("grey.6"),
                    "aria_label is required, not optional - a QR code conveys real "
                    "information to a sighted/scanning user, but nothing to a screen "
                    "reader without one.",
                }
            }
        }
    }
}
