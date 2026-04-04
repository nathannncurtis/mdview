#![windows_subsystem = "windows"]

use notify::{EventKind, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::{env, fs, path::PathBuf, process};

static LOG_FILE: Mutex<Option<fs::File>> = Mutex::new(None);

fn init_log() {
    if let Some(dir) = dirs_next() {
        let log_dir = dir.join("mdview");
        let _ = fs::create_dir_all(&log_dir);
        let path = log_dir.join("mdview.log");
        if let Ok(f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            *LOG_FILE.lock().unwrap() = Some(f);
        }
    }
}

fn log(msg: &str) {
    if let Ok(mut guard) = LOG_FILE.lock() {
        if let Some(ref mut f) = *guard {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let _ = writeln!(f, "[{now}] {msg}");
            let _ = f.flush();
        }
    }
}
use tao::dpi::LogicalSize;
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

fn main() {
    init_log();
    std::panic::set_hook(Box::new(|info| {
        log(&format!("PANIC: {info}"));
    }));
    let args: Vec<String> = env::args().collect();
    log(&format!("started with args: {:?}", args));

    if args.len() >= 2 && args[1] == "--register" {
        register_file_association();
        return;
    }

    if args.len() < 2 {
        eprintln!("Usage: mdview <file.md>");
        eprintln!("       mdview --register");
        process::exit(1);
    }

    let path = fs::canonicalize(&args[1]).unwrap_or_else(|e| {
        log(&format!("failed to resolve path {}: {e}", &args[1]));
        process::exit(1);
    });
    // Strip \\?\ prefix that Windows canonicalize adds
    let path = PathBuf::from(path.to_string_lossy().trim_start_matches("\\\\?\\").to_string());
    log(&format!("opening: {}", path.display()));

    let markdown = fs::read_to_string(&path).unwrap_or_else(|e| {
        log(&format!("failed to read {}: {e}", path.display()));
        process::exit(1);
    });
    log(&format!("read {} bytes", markdown.len()));

    let filename = path
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("mdview");

    let base_dir = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    let full_html = render_markdown(&markdown, base_dir);
    let scroll_key = scroll_key_for(&path);
    let saved_scroll = load_scroll_position(&scroll_key);

    let html_content: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(full_html.into_bytes()));
    log("rendering html via custom protocol");

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(format!("{filename} — mdview"))
        .with_inner_size(LogicalSize::new(900.0, 700.0))
        .with_decorations(false)
        .build(&event_loop)
        .expect("Failed to create window");

    let scroll_key_ipc = scroll_key.clone();
    let proxy2 = proxy.clone();
    let html_for_protocol = Arc::clone(&html_content);
    let webview = WebViewBuilder::new()
        .with_custom_protocol("mdview".into(), move |_id, _request| {
            let body = html_for_protocol.lock().unwrap().clone();
            wry::http::Response::builder()
                .header("Content-Type", "text/html; charset=utf-8")
                .body(body.into())
                .unwrap()
        })
        .with_url("mdview://localhost")
        .with_initialization_script(&format!(
            r#"window.addEventListener('load', function() {{ window.scrollTo(0, {saved_scroll}); }});
            setInterval(function() {{ window.ipc.postMessage('scroll:' + window.scrollY); }}, 2000);"#
        ))
        .with_initialization_script(DRAG_RESIZE_JS)
        .with_ipc_handler(move |msg| {
            let msg = msg.body();
            if let Some(pos_str) = msg.strip_prefix("scroll:") {
                if let Ok(pos) = pos_str.parse::<f64>() {
                    save_scroll_position(&scroll_key_ipc, pos);
                }
            } else if msg == "drag" {
                let _ = proxy2.send_event(UserEvent::DragWindow);
            } else if msg == "close" {
                let _ = proxy2.send_event(UserEvent::CloseWindow);
            } else if let Some(url) = msg.strip_prefix("open:") {
                let _ = open::that(url);
            }
        })
        .build(&window)
        .expect("Failed to create webview");

    log("webview created");
    log("setting up file watcher");

    // File watcher
    let watch_path = path.clone();
    let _watcher = {
        let proxy = proxy.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<notify::Event, _>| {
            if let Ok(event) = res {
                if matches!(event.kind, EventKind::Modify(_)) {
                    let _ = proxy.send_event(UserEvent::FileChanged);
                }
            }
        })
        .ok();
        if let Some(ref mut w) = watcher {
            let _ = w.watch(watch_path.as_ref(), RecursiveMode::NonRecursive);
        }
        watcher
    };

    log("file watcher ready");

    let reload_path = path.clone();
    log("entering event loop");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::DroppedFile(ref dropped_path),
                ..
            } => {
                log(&format!("file dropped: {}", dropped_path.display()));
                if dropped_path
                    .extension()
                    .map_or(false, |e| e == "md" || e == "markdown")
                {
                    if let Ok(md) = fs::read_to_string(&dropped_path) {
                        let drop_base = dropped_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                        let html = render_markdown(&md, drop_base);
                        *html_content.lock().unwrap() = html.into_bytes();
                        let _ = webview.load_url("mdview://localhost");
                        let title = dropped_path
                            .file_name()
                            .and_then(|f| f.to_str())
                            .unwrap_or("mdview");
                        window.set_title(&format!("{title} — mdview"));
                    }
                }
            }

            Event::UserEvent(UserEvent::FileChanged) => {
                log("file changed, reloading");
                if let Ok(md) = fs::read_to_string(&reload_path) {
                    let reload_base = reload_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                    let html = render_markdown(&md, reload_base);
                    *html_content.lock().unwrap() = html.into_bytes();
                    let _ = webview.load_url("mdview://localhost");
                }
            }

            Event::UserEvent(UserEvent::DragWindow) => {
                let _ = window.drag_window();
            }

            Event::UserEvent(UserEvent::CloseWindow) => {
                log("closing");
                *control_flow = ControlFlow::Exit;
            }

            _ => {}
        }
    });
}

#[derive(Debug)]
enum UserEvent {
    FileChanged,
    DragWindow,
    CloseWindow,
}

fn render_markdown(markdown: &str, base_dir: &std::path::Path) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(markdown, opts);

    let mut html_body = String::new();
    html::push_html(&mut html_body, parser);

    // Rewrite relative src= paths to absolute file:// URLs
    let base_url = format!("file:///{}", base_dir.to_string_lossy().replace('\\', "/"));
    let mut result = String::with_capacity(html_body.len());
    let mut rest = html_body.as_str();
    while let Some(idx) = rest.find("src=\"") {
        result.push_str(&rest[..idx]);
        let after = &rest[idx + 5..]; // after src="
        if after.starts_with("http://")
            || after.starts_with("https://")
            || after.starts_with("file://")
            || after.starts_with("data:")
            || after.starts_with('/')
        {
            result.push_str("src=\"");
        } else {
            result.push_str(&format!("src=\"{base_url}/"));
        }
        rest = after;
    }
    result.push_str(rest);
    html_body = result;

    format!(
        r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><style>
{CSS}
{HIGHLIGHT_CSS}
</style></head><body><article>{html_body}</article>
<script>{HIGHLIGHT_JS}</script>
</body></html>"#
    )
}

// -- Scroll position persistence --

fn scroll_key_for(path: &PathBuf) -> String {
    path.to_string_lossy().to_string()
}

fn scroll_data_path() -> Option<PathBuf> {
    let mut dir = dirs_next()?;
    dir.push("mdview");
    let _ = fs::create_dir_all(&dir);
    dir.push("scroll.json");
    Some(dir)
}

fn dirs_next() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

fn load_scroll_positions() -> HashMap<String, f64> {
    let Some(path) = scroll_data_path() else {
        return HashMap::new();
    };
    let Ok(data) = fs::read_to_string(&path) else {
        return HashMap::new();
    };
    parse_scroll_json(&data)
}

fn load_scroll_position(key: &str) -> f64 {
    load_scroll_positions().get(key).copied().unwrap_or(0.0)
}

fn save_scroll_position(key: &str, pos: f64) {
    let mut positions = load_scroll_positions();
    if pos < 1.0 {
        positions.remove(key);
    } else {
        positions.insert(key.to_string(), pos);
    }
    if let Some(path) = scroll_data_path() {
        let json = serialize_scroll_json(&positions);
        let _ = fs::write(path, json);
    }
}

fn parse_scroll_json(data: &str) -> HashMap<String, f64> {
    let mut map = HashMap::new();
    let data = data.trim();
    if !data.starts_with('{') || !data.ends_with('}') {
        return map;
    }
    let inner = &data[1..data.len() - 1];
    for pair in inner.split(',') {
        let pair = pair.trim();
        if let Some((k, v)) = pair.split_once(':') {
            let k = k.trim().trim_matches('"');
            if let Ok(v) = v.trim().parse::<f64>() {
                map.insert(k.to_string(), v);
            }
        }
    }
    map
}

fn serialize_scroll_json(map: &HashMap<String, f64>) -> String {
    let mut s = String::from("{");
    for (i, (k, v)) in map.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str(&format!("\"{}\":{}", k.replace('\\', "\\\\").replace('"', "\\\""), v));
    }
    s.push('}');
    s
}

// -- File association registry --

fn register_file_association() {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let exe_path = env::current_exe().expect("Failed to get executable path");
        let exe_str = exe_path.to_string_lossy();

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);

        // Register the application
        let (app_key, _) = hkcu
            .create_subkey("Software\\Classes\\mdview")
            .expect("Failed to create registry key");
        app_key
            .set_value("", &"Markdown File")
            .expect("Failed to set value");

        let (icon_key, _) = app_key
            .create_subkey("DefaultIcon")
            .expect("Failed to create icon key");
        icon_key
            .set_value("", &format!("{exe_str},0"))
            .expect("Failed to set icon");

        let (cmd_key, _) = app_key
            .create_subkey("shell\\open\\command")
            .expect("Failed to create command key");
        cmd_key
            .set_value("", &format!("\"{exe_str}\" \"%1\""))
            .expect("Failed to set command");

        // Associate .md extension
        let (ext_key, _) = hkcu
            .create_subkey("Software\\Classes\\.md")
            .expect("Failed to create .md key");
        ext_key
            .set_value("", &"mdview")
            .expect("Failed to set .md association");

        // Associate .markdown extension
        let (ext_key2, _) = hkcu
            .create_subkey("Software\\Classes\\.markdown")
            .expect("Failed to create .markdown key");
        ext_key2
            .set_value("", &"mdview")
            .expect("Failed to set .markdown association");

        println!("mdview registered as default viewer for .md and .markdown files.");
        println!("You may need to restart Explorer or log out/in for changes to take effect.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("--register is only supported on Windows");
        process::exit(1);
    }
}

const CSS: &str = r#"
    * { margin: 0; padding: 0; box-sizing: border-box; }
    body {
        font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
        font-size: 16px;
        line-height: 1.6;
        color: #c9d1d9;
        background: #0d1117;
        padding: 40px;
        max-width: 900px;
        margin: 0 auto;
    }
    article > *:first-child { margin-top: 0; }
    h1, h2, h3, h4, h5, h6 { margin-top: 24px; margin-bottom: 16px; font-weight: 600; line-height: 1.25; }
    h1 { font-size: 2em; padding-bottom: 0.3em; border-bottom: 1px solid #21262d; }
    h2 { font-size: 1.5em; padding-bottom: 0.3em; border-bottom: 1px solid #21262d; }
    h3 { font-size: 1.25em; }
    p { margin-top: 0; margin-bottom: 16px; }
    a { color: #58a6ff; text-decoration: none; }
    a:hover { text-decoration: underline; }
    code {
        font-family: "SFMono-Regular", Consolas, "Liberation Mono", Menlo, monospace;
        font-size: 85%;
        background: #161b22;
        padding: 0.2em 0.4em;
        border-radius: 3px;
    }
    pre {
        background: #161b22;
        padding: 16px;
        border-radius: 6px;
        overflow-x: auto;
        margin-bottom: 16px;
        line-height: 1.45;
    }
    pre code { background: none; padding: 0; font-size: 85%; }
    blockquote {
        padding: 0 1em;
        color: #8b949e;
        border-left: 0.25em solid #30363d;
        margin-bottom: 16px;
    }
    ul, ol { padding-left: 2em; margin-bottom: 16px; }
    li + li { margin-top: 0.25em; }
    table { border-collapse: collapse; margin-bottom: 16px; width: auto; }
    th, td { padding: 6px 13px; border: 1px solid #30363d; }
    th { font-weight: 600; background: #161b22; }
    tr:nth-child(2n) { background: #161b22; }
    hr { height: 0.25em; padding: 0; margin: 24px 0; background: #21262d; border: 0; }
    img { max-width: 100%; }
    input[type="checkbox"] { margin-right: 0.5em; }
    #mdview-close {
        position: fixed;
        top: 8px;
        right: 12px;
        width: 28px;
        height: 28px;
        display: flex;
        align-items: center;
        justify-content: center;
        cursor: pointer;
        color: #8b949e;
        font-size: 14px;
        border-radius: 4px;
        z-index: 9999;
        user-select: none;
        transition: background 0.15s, color 0.15s;
    }
    #mdview-close:hover { background: #da3633; color: #fff; }
"#;

const HIGHLIGHT_CSS: &str = r#"
    .hl-keyword { color: #ff7b72; }
    .hl-string { color: #a5d6ff; }
    .hl-comment { color: #8b949e; font-style: italic; }
    .hl-number { color: #79c0ff; }
    .hl-type { color: #ffa657; }
    .hl-func { color: #d2a8ff; }
    .hl-punct { color: #c9d1d9; }
    .hl-attr { color: #7ee787; }
    .hl-bool { color: #79c0ff; }
"#;

const HIGHLIGHT_JS: &str = r#"
(function() {
    var LANGS = {
        rust: {
            keywords: /\b(as|async|await|break|const|continue|crate|dyn|else|enum|extern|fn|for|if|impl|in|let|loop|match|mod|move|mut|pub|ref|return|self|Self|static|struct|super|trait|type|union|unsafe|use|where|while|yield)\b/g,
            types: /\b(bool|char|f32|f64|i8|i16|i32|i64|i128|isize|str|u8|u16|u32|u64|u128|usize|String|Vec|Option|Result|Box|Rc|Arc|HashMap|HashSet)\b/g,
            bools: /\b(true|false|None|Some|Ok|Err)\b/g,
        },
        python: {
            keywords: /\b(and|as|assert|async|await|break|class|continue|def|del|elif|else|except|finally|for|from|global|if|import|in|is|lambda|nonlocal|not|or|pass|raise|return|try|while|with|yield)\b/g,
            types: /\b(int|float|str|bool|list|dict|tuple|set|bytes|None|True|False)\b/g,
            bools: /\b(True|False|None)\b/g,
        },
        javascript: {
            keywords: /\b(async|await|break|case|catch|class|const|continue|debugger|default|delete|do|else|export|extends|finally|for|from|function|if|import|in|instanceof|let|new|of|return|super|switch|this|throw|try|typeof|var|void|while|with|yield)\b/g,
            types: /\b(Array|Boolean|Date|Error|Function|Map|Number|Object|Promise|RegExp|Set|String|Symbol|WeakMap|WeakSet)\b/g,
            bools: /\b(true|false|null|undefined|NaN|Infinity)\b/g,
        },
        go: {
            keywords: /\b(break|case|chan|const|continue|default|defer|else|fallthrough|for|func|go|goto|if|import|interface|map|package|range|return|select|struct|switch|type|var)\b/g,
            types: /\b(bool|byte|complex64|complex128|error|float32|float64|int|int8|int16|int32|int64|rune|string|uint|uint8|uint16|uint32|uint64|uintptr)\b/g,
            bools: /\b(true|false|nil|iota)\b/g,
        },
        json: {
            bools: /\b(true|false|null)\b/g,
        },
    };
    LANGS.js = LANGS.javascript;
    LANGS.ts = LANGS.javascript;
    LANGS.typescript = LANGS.javascript;
    LANGS.rs = LANGS.rust;
    LANGS.py = LANGS.python;
    LANGS.golang = LANGS.go;

    function esc(s) {
        return s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
    }

    function highlight(code, lang) {
        var tokens = [];
        var re = /(\/\/[^\n]*|\/\*[\s\S]*?\*\/|#[^\n]*|"""[\s\S]*?"""|'''[\s\S]*?'''|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\b\d+\.?\d*(?:e[+-]?\d+)?\b)/gi;
        var last = 0;
        var m;
        while ((m = re.exec(code)) !== null) {
            if (m.index > last) tokens.push({t:'code', v:code.slice(last, m.index)});
            var v = m[0];
            if (v[0]==='/' && (v[1]==='/' || v[1]==='*') || v[0]==='#' && lang !== 'rust') {
                tokens.push({t:'comment', v:v});
            } else if (v[0]==='"' || v[0]==="'" || v[0]==='`') {
                tokens.push({t:'string', v:v});
            } else {
                tokens.push({t:'number', v:v});
            }
            last = re.lastIndex;
        }
        if (last < code.length) tokens.push({t:'code', v:code.slice(last)});

        var defs = LANGS[lang] || {};
        var out = '';
        for (var i = 0; i < tokens.length; i++) {
            var tk = tokens[i];
            if (tk.t !== 'code') {
                out += '<span class="hl-' + tk.t + '">' + esc(tk.v) + '</span>';
            } else {
                var s = esc(tk.v);
                if (defs.bools) s = s.replace(defs.bools, '<span class="hl-bool">$&</span>');
                if (defs.types) s = s.replace(defs.types, '<span class="hl-type">$&</span>');
                if (defs.keywords) s = s.replace(defs.keywords, '<span class="hl-keyword">$&</span>');
                s = s.replace(/([a-zA-Z_]\w*)\s*(?=\()/g, '<span class="hl-func">$1</span>');
                out += s;
            }
        }
        return out;
    }

    document.querySelectorAll('pre code').forEach(function(el) {
        var cls = el.className || '';
        var lang = (cls.match(/language-(\w+)/) || [])[1] || '';
        if (lang || el.textContent.length > 0) {
            el.innerHTML = highlight(el.textContent, lang.toLowerCase());
        }
    });
})();
"#;

const DRAG_RESIZE_JS: &str = r#"
document.addEventListener('DOMContentLoaded', function() {
    // Close button
    var btn = document.createElement('div');
    btn.id = 'mdview-close';
    btn.innerHTML = '&#x2715;';
    btn.addEventListener('click', function() { window.ipc.postMessage('close'); });
    document.body.appendChild(btn);

    // Alt+drag from anywhere to move window
    document.addEventListener('mousedown', function(e) {
        if (e.button !== 0 || !e.altKey) return;
        e.preventDefault();
        window.ipc.postMessage('drag');
    });

    // Ctrl+Q to quit
    document.addEventListener('keydown', function(e) {
        if (e.ctrlKey && e.key === 'q') {
            e.preventDefault();
            window.ipc.postMessage('close');
        }
    });

    // Make links open in default browser
    document.addEventListener('click', function(e) {
        var a = e.target.closest('a');
        if (a && a.href && !a.href.startsWith('about:')) {
            e.preventDefault();
            window.ipc.postMessage('open:' + a.href);
        }
    });
});
"#;
