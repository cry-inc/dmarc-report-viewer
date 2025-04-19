use axum::extract::Request;
use axum::http::header;
use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn handler(req: Request) -> impl IntoResponse {
    let path = req.uri().path();
    for sf in STATIC_FILES {
        if sf.http_path == path {
            let mut mime_type = "application/octet-stream";
            for mt in MIME_TYPES {
                if sf.file_path.ends_with(mt.ext) {
                    mime_type = mt.mime_type;
                    break;
                }
            }
            return (
                StatusCode::OK,
                [(header::CONTENT_TYPE, mime_type)],
                #[cfg(debug_assertions)]
                std::fs::read(sf.file_path).expect("Failed to read file"),
                #[cfg(not(debug_assertions))]
                sf._data,
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
        _data: include_bytes!("../../ui/index.html"),
    },
    StaticFile {
        http_path: "/chart.js",
        file_path: "ui/chart.umd.4.4.2.min.js",
        _data: include_bytes!("../../ui/chart.umd.4.4.2.min.js"),
    },
    StaticFile {
        http_path: "/lit.js",
        file_path: "ui/lit-core.3.1.4.min.js",
        _data: include_bytes!("../../ui/lit-core.3.1.4.min.js"),
    },
    StaticFile {
        http_path: "/components/style.js",
        file_path: "ui/components/style.js",
        _data: include_bytes!("../../ui/components/style.js"),
    },
    StaticFile {
        http_path: "/components/app.js",
        file_path: "ui/components/app.js",
        _data: include_bytes!("../../ui/components/app.js"),
    },
    StaticFile {
        http_path: "/components/dashboard.js",
        file_path: "ui/components/dashboard.js",
        _data: include_bytes!("../../ui/components/dashboard.js"),
    },
    StaticFile {
        http_path: "/components/mailtable.js",
        file_path: "ui/components/mailtable.js",
        _data: include_bytes!("../../ui/components/mailtable.js"),
    },
    StaticFile {
        http_path: "/components/report.js",
        file_path: "ui/components/report.js",
        _data: include_bytes!("../../ui/components/report.js"),
    },
    StaticFile {
        http_path: "/components/reports.js",
        file_path: "ui/components/reports.js",
        _data: include_bytes!("../../ui/components/reports.js"),
    },
    StaticFile {
        http_path: "/components/mails.js",
        file_path: "ui/components/mails.js",
        _data: include_bytes!("../../ui/components/mails.js"),
    },
    StaticFile {
        http_path: "/components/mail.js",
        file_path: "ui/components/mail.js",
        _data: include_bytes!("../../ui/components/mail.js"),
    },
    StaticFile {
        http_path: "/components/about.js",
        file_path: "ui/components/about.js",
        _data: include_bytes!("../../ui/components/about.js"),
    },
    StaticFile {
        http_path: "/components/reporttable.js",
        file_path: "ui/components/reporttable.js",
        _data: include_bytes!("../../ui/components/reporttable.js"),
    },
];

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

struct MimeType {
    ext: &'static str,
    mime_type: &'static str,
}

struct StaticFile {
    http_path: &'static str,
    file_path: &'static str,
    _data: &'static [u8],
}
