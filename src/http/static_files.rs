use axum::extract::Request;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;

pub async fn handler(req: Request) -> impl IntoResponse {
    let path = req.uri().path();
    for sf in STATIC_FILES {
        if sf.http_path == path {
            let mime_type = MimeType::from_path(sf.file_path);
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                #[cfg(debug_assertions)]
                {
                    // During debug builds we first try to load the file from the checkout folder.
                    // If that does not work, we fall back to the embeddef file from the binary.
                    tokio::fs::read(sf.file_path)
                        .await
                        .unwrap_or(sf.data.to_vec())
                },
                #[cfg(not(debug_assertions))]
                {
                    // During release builds we always use the files embedded into the binary!
                    sf.data
                },
            );
        }
    }
    (
        StatusCode::NOT_FOUND,
        [(header::CONTENT_TYPE, "text/plain")],
        #[cfg(debug_assertions)]
        b"File not found".to_vec(),
        #[cfg(not(debug_assertions))]
        b"File not found",
    )
}

const STATIC_FILES: &[StaticFile] = &[
    StaticFile {
        http_path: "/",
        file_path: "ui/index.html",
        data: include_bytes!("../../ui/index.html"),
    },
    StaticFile {
        http_path: "/chart.js",
        file_path: "ui/chart.umd.4.5.0.min.js",
        data: include_bytes!("../../ui/chart.umd.4.5.0.min.js"),
    },
    StaticFile {
        http_path: "/lit.js",
        file_path: "ui/lit-core.3.3.0.min.js",
        data: include_bytes!("../../ui/lit-core.3.3.0.min.js"),
    },
    StaticFile {
        http_path: "/style.js",
        file_path: "ui/style.js",
        data: include_bytes!("../../ui/style.js"),
    },
    StaticFile {
        http_path: "/utils.js",
        file_path: "ui/utils.js",
        data: include_bytes!("../../ui/utils.js"),
    },
    StaticFile {
        http_path: "/components/app.js",
        file_path: "ui/components/app.js",
        data: include_bytes!("../../ui/components/app.js"),
    },
    StaticFile {
        http_path: "/components/dashboard.js",
        file_path: "ui/components/dashboard.js",
        data: include_bytes!("../../ui/components/dashboard.js"),
    },
    StaticFile {
        http_path: "/components/mail-table.js",
        file_path: "ui/components/mail-table.js",
        data: include_bytes!("../../ui/components/mail-table.js"),
    },
    StaticFile {
        http_path: "/components/dmarc-report.js",
        file_path: "ui/components/dmarc-report.js",
        data: include_bytes!("../../ui/components/dmarc-report.js"),
    },
    StaticFile {
        http_path: "/components/tls-report.js",
        file_path: "ui/components/tls-report.js",
        data: include_bytes!("../../ui/components/tls-report.js"),
    },
    StaticFile {
        http_path: "/components/dmarc-reports.js",
        file_path: "ui/components/dmarc-reports.js",
        data: include_bytes!("../../ui/components/dmarc-reports.js"),
    },
    StaticFile {
        http_path: "/components/tls-reports.js",
        file_path: "ui/components/tls-reports.js",
        data: include_bytes!("../../ui/components/tls-reports.js"),
    },
    StaticFile {
        http_path: "/components/mails.js",
        file_path: "ui/components/mails.js",
        data: include_bytes!("../../ui/components/mails.js"),
    },
    StaticFile {
        http_path: "/components/mail.js",
        file_path: "ui/components/mail.js",
        data: include_bytes!("../../ui/components/mail.js"),
    },
    StaticFile {
        http_path: "/components/sources.js",
        file_path: "ui/components/sources.js",
        data: include_bytes!("../../ui/components/sources.js"),
    },
    StaticFile {
        http_path: "/components/about.js",
        file_path: "ui/components/about.js",
        data: include_bytes!("../../ui/components/about.js"),
    },
    StaticFile {
        http_path: "/components/dmarc-report-table.js",
        file_path: "ui/components/dmarc-report-table.js",
        data: include_bytes!("../../ui/components/dmarc-report-table.js"),
    },
    StaticFile {
        http_path: "/components/tls-report-table.js",
        file_path: "ui/components/tls-report-table.js",
        data: include_bytes!("../../ui/components/tls-report-table.js"),
    },
];

struct MimeType {
    ext: &'static str,
    mime_type: &'static str,
}

impl MimeType {
    fn from_path(file_path: &str) -> &'static str {
        for mt in MIME_TYPES {
            if file_path.ends_with(mt.ext) {
                return mt.mime_type;
            }
        }
        "application/octet-stream"
    }
}

const MIME_TYPES: &[MimeType] = &[
    MimeType {
        ext: ".html",
        mime_type: "text/html",
    },
    MimeType {
        ext: ".js",
        mime_type: "text/javascript",
    },
    MimeType {
        ext: ".css",
        mime_type: "text/css",
    },
];

struct StaticFile {
    http_path: &'static str,
    file_path: &'static str,
    data: &'static [u8],
}
