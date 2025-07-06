use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

#[cfg(target_os = "emscripten")]
use std::ffi::CString;
#[cfg(target_os = "emscripten")]
use std::os::raw::{c_char, c_int, c_void};
#[cfg(target_os = "emscripten")]
use futures::channel::oneshot;

// Emscripten 运行时现在由 async-runtime crate 提供


// Emscripten FFI 绑定
#[cfg(target_os = "emscripten")]
extern "C" {
    fn emscripten_fetch_attr_init(attr: *mut EmscriptenFetchAttributes);
    fn emscripten_fetch(attr: *const EmscriptenFetchAttributes, url: *const c_char) -> *mut EmscriptenFetch;
    fn emscripten_fetch_close(fetch: *mut EmscriptenFetch) -> c_int;
}

#[cfg(target_os = "emscripten")]
const EMSCRIPTEN_FETCH_LOAD_TO_MEMORY: u32 = 1;

#[cfg(target_os = "emscripten")]
type EmscriptenFetchCallback = unsafe extern "C" fn(*mut EmscriptenFetch);

#[cfg(target_os = "emscripten")]
#[repr(C)]
struct EmscriptenFetchAttributes {
    request_method: [c_char; 32],
    user_data: *mut c_void,
    on_success: *const EmscriptenFetchCallback,
    on_error: *const EmscriptenFetchCallback,
    on_progress: *const EmscriptenFetchCallback,
    on_ready_state_change: *const EmscriptenFetchCallback,
    attributes: u32,
    timeout_msecs: u32,
    with_credentials: u8,
    destination_path: *const c_char,
    user_name: *const c_char,
    password: *const c_char,
    request_headers: *const *const c_char,
    overridden_mime_type: *const c_char,
    request_data: *const c_char,
    request_data_size: usize,
}

#[cfg(target_os = "emscripten")]
#[repr(C)]
struct EmscriptenFetch {
    id: u32,
    user_data: *mut c_void,
    url: *const c_char,
    data: *const c_char,
    num_bytes: u64,
    data_offset: u64,
    total_bytes: u64,
    ready_state: u16,
    status: u16,
    status_text: [c_char; 64],
    __attributes: EmscriptenFetchAttributes,
    response_url: *const c_char,
}

// 公共类型定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchOptions {
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timeout_ms: Option<u32>,
}

impl Default for FetchOptions {
    fn default() -> Self {
        Self {
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            timeout_ms: Some(30000),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    pub status: u16,
    pub status_text: String,
    pub data: Vec<u8>,
    pub url: String,
}

#[derive(Debug, Clone)]
pub enum FetchError {
    NetworkError(String),
    InvalidUrl(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            FetchError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
        }
    }
}

impl std::error::Error for FetchError {}

pub type FetchResult<T> = Result<T, FetchError>;

// Future 执行器 - 使用 async-runtime 提供的实现
#[cfg(not(target_os = "emscripten"))]
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    async_runtime::spawn(future);
}

#[cfg(target_os = "emscripten")]
pub fn spawn_local<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    // 在 Emscripten 中，spawn 不需要 Send 约束
    async_runtime::spawn(future);
}

// 核心 Future 封装
pub struct FetchFuture {
    #[cfg(target_os = "emscripten")]
    receiver: oneshot::Receiver<FetchResult<FetchResponse>>,
    #[cfg(not(target_os = "emscripten"))]
    _phantom: std::marker::PhantomData<()>,
}

impl Future for FetchFuture {
    type Output = FetchResult<FetchResponse>;

    #[cfg(target_os = "emscripten")]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx).map(|result| {
            result.unwrap_or_else(|_| Err(FetchError::NetworkError("Channel closed".to_string())))
        })
    }

    #[cfg(not(target_os = "emscripten"))]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(Err(FetchError::NetworkError(
            "Fetch only supported on Emscripten".to_string(),
        )))
    }
}

// 回调函数
#[cfg(target_os = "emscripten")]
unsafe extern "C" fn on_success(fetch: *mut EmscriptenFetch) {
    if fetch.is_null() {
        return;
    }

    let fetch_ref = &*fetch;
    let user_data = fetch_ref.user_data;

    if !user_data.is_null() {
        // 提取响应数据
        let data = if !fetch_ref.data.is_null() && fetch_ref.num_bytes > 0 {
            let size = std::cmp::min(fetch_ref.num_bytes as usize, 10 * 1024 * 1024);
            let mut data = vec![0u8; size];
            std::ptr::copy_nonoverlapping(fetch_ref.data as *const u8, data.as_mut_ptr(), size);
            data
        } else {
            Vec::new()
        };

        // 获取状态文本
        let status_text_len = fetch_ref.status_text.iter().position(|&c| c == 0).unwrap_or(63);
        let status_text = if status_text_len > 0 {
            let status_bytes: Vec<u8> = fetch_ref.status_text[..status_text_len]
                .iter()
                .map(|&c| c as u8)
                .collect();
            String::from_utf8_lossy(&status_bytes).to_string()
        } else {
            String::new()
        };

        // 获取 URL
        let url = if !fetch_ref.url.is_null() {
            std::ffi::CStr::from_ptr(fetch_ref.url)
                .to_string_lossy()
                .to_string()
        } else {
            "unknown".to_string()
        };

        let response = FetchResponse {
            status: fetch_ref.status as u16,
            status_text,
            data,
            url,
        };

        // 发送结果
        let sender: Box<oneshot::Sender<FetchResult<FetchResponse>>> = Box::from_raw(user_data as *mut oneshot::Sender<FetchResult<FetchResponse>>);
        let _ = sender.send(Ok(response));
    }

    emscripten_fetch_close(fetch);
}

#[cfg(target_os = "emscripten")]
unsafe extern "C" fn on_error(fetch: *mut EmscriptenFetch) {
    if fetch.is_null() {
        return;
    }

    let fetch_ref = &*fetch;
    let user_data = fetch_ref.user_data;

    if !user_data.is_null() {
        let error_msg = if fetch_ref.status == 0 {
            "Network error".to_string()
        } else {
            format!("HTTP error: {}", fetch_ref.status)
        };

        let sender: Box<oneshot::Sender<FetchResult<FetchResponse>>> = Box::from_raw(user_data as *mut oneshot::Sender<FetchResult<FetchResponse>>);
        let _ = sender.send(Err(FetchError::NetworkError(error_msg)));
    }

    emscripten_fetch_close(fetch);
}

// 主要 API
pub struct WebFetch;

impl WebFetch {
    pub fn fetch(url: &str, options: Option<FetchOptions>) -> FetchFuture {
        #[cfg(target_os = "emscripten")]
        {
            let options = options.unwrap_or_default();
            let (sender, receiver) = oneshot::channel();

            let c_url = match CString::new(url) {
                Ok(url) => url,
                Err(_) => {
                    let _ = sender.send(Err(FetchError::InvalidUrl("Invalid URL".to_string())));
                    return FetchFuture { receiver };
                }
            };

            unsafe {
                let mut attr: EmscriptenFetchAttributes = std::mem::zeroed();
                emscripten_fetch_attr_init(&mut attr);

                attr.attributes = EMSCRIPTEN_FETCH_LOAD_TO_MEMORY;
                attr.timeout_msecs = options.timeout_ms.unwrap_or(30000);
                attr.with_credentials = 0;
                attr.on_success = on_success as *const EmscriptenFetchCallback;
                attr.on_error = on_error as *const EmscriptenFetchCallback;
                attr.user_data = Box::into_raw(Box::new(sender)) as *mut c_void;

                // 设置请求方法
                let method_bytes = options.method.as_bytes();
                let method_len = std::cmp::min(method_bytes.len(), 31);
                std::ptr::copy_nonoverlapping(
                    method_bytes.as_ptr() as *const c_char,
                    attr.request_method.as_mut_ptr(),
                    method_len,
                );
                if method_len < 31 {
                    attr.request_method[method_len] = 0;
                }

                // 设置请求体
                if let Some(body) = &options.body {
                    if let Ok(body_cstring) = CString::new(body.as_bytes()) {
                        attr.request_data = body_cstring.as_ptr();
                        attr.request_data_size = body_cstring.as_bytes().len();
                        std::mem::forget(body_cstring);
                    }
                }

                let fetch = emscripten_fetch(&attr, c_url.as_ptr());
                if fetch.is_null() {
                    let sender: Box<oneshot::Sender<FetchResult<FetchResponse>>> = Box::from_raw(attr.user_data as *mut oneshot::Sender<FetchResult<FetchResponse>>);
                    let _ = sender.send(Err(FetchError::NetworkError("Failed to create request".to_string())));
                }
            }

            FetchFuture { receiver }
        }

        #[cfg(not(target_os = "emscripten"))]
        {
            let _ = (url, options);
            FetchFuture { _phantom: std::marker::PhantomData }
        }
    }

    pub fn get(url: &str) -> FetchFuture {
        Self::fetch(url, None)
    }

    pub fn post(url: &str, body: &str) -> FetchFuture {
        let options = FetchOptions {
            method: "POST".to_string(),
            body: Some(body.to_string()),
            ..Default::default()
        };
        Self::fetch(url, Some(options))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fetch_options_default() {
        let options = FetchOptions::default();
        assert_eq!(options.method, "GET");
        assert_eq!(options.timeout_ms, Some(30000));
        assert!(options.headers.is_empty());
        assert!(options.body.is_none());
    }

    #[test]
    fn test_fetch_error_display() {
        let error = FetchError::NetworkError("Test error".to_string());
        assert_eq!(format!("{}", error), "Network error: Test error");
    }
}