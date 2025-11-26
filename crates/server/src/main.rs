use azookey_server::TonicNamedPipeServer;
use tonic::{transport::Server, Request, Response, Status};
use tonic_reflection::server::Builder as ReflectionBuilder;

use shared::proto::azookey_service_server::{AzookeyService, AzookeyServiceServer};
use shared::proto::{
    AppendTextRequest, AppendTextResponse, ClearTextRequest, ClearTextResponse, ComposingText,
    MoveCursorRequest, MoveCursorResponse, RemoveTextRequest, RemoveTextResponse,
    ShrinkTextRequest, ShrinkTextResponse, Suggestion,
    // いい感じ変換
    IsIikanjiKeywordRequest, IsIikanjiKeywordResponse,
    IsIikanjiEnabledRequest, IsIikanjiEnabledResponse,
    RequestIikanjiRequest, RequestIikanjiResponse,
};

use std::ffi::{c_char, c_int, CStr, CString};

const USE_ZENZAI: bool = true;

struct RawComposingText {
    text: String,
    cursor: i8,
}

#[derive(Debug, Clone)]
#[repr(C)]
struct FFICandidate {
    text: *mut c_char,
    subtext: *mut c_char,
    hiragana: *mut c_char,
    corresponding_count: c_int,
}

unsafe extern "C" {
    fn Initialize(path: *const c_char, use_zenzai: bool);
    fn SetContext(context: *const c_char);
    fn AppendText(input: *const c_char, cursorPtr: *mut c_int) -> *mut c_char;
    fn RemoveText(cursorPtr: *mut c_int) -> *mut c_char;
    fn MoveCursor(offset: c_int, cursorPtr: *mut c_int) -> *mut c_char;
    fn ShrinkText(offset: c_int) -> *mut c_char;
    fn ClearText();
    fn GetComposedText(lengthPtr: *mut c_int) -> *mut *mut FFICandidate;
    fn LoadConfig();
    
    // いい感じ変換FFI関数
    fn IsIikanjiKeyword(input: *const c_char) -> bool;
    fn IsIikanjiEnabled() -> bool;
    fn RequestIikanji(keyword: *const c_char, context: *const c_char) -> *mut c_char;
}

fn initialize(path: &str) {
    unsafe {
        let path = CString::new(path).expect("CString::new failed");
        Initialize(path.as_ptr(), USE_ZENZAI);
    }
}

fn add_text(input: &str) -> RawComposingText {
    unsafe {
        let input = CString::new(input).expect("CString::new failed");
        let mut cursor: c_int = 0;

        let result = AppendText(input.as_ptr(), &mut cursor);

        let text = CStr::from_ptr(&*result as *const c_char).to_str().unwrap();

        RawComposingText {
            text: text.to_string(),
            cursor: cursor as i8,
        }
    }
}

fn move_cursor(offset: i8) -> RawComposingText {
    unsafe {
        let offset = c_int::from(offset);
        println!("Offset: {}", offset);
        let mut cursor: c_int = 0;

        let result = MoveCursor(offset, &mut cursor);

        let text = CStr::from_ptr(&*result as *const c_char).to_str().unwrap();

        RawComposingText {
            text: text.to_string(),
            cursor: cursor as i8,
        }
    }
}

fn remove_text() -> RawComposingText {
    unsafe {
        let mut cursor: c_int = 0;

        let result = RemoveText(&mut cursor);

        let text = CStr::from_ptr(&*result as *const c_char).to_str().unwrap();

        RawComposingText {
            text: text.to_string(),
            cursor: cursor as i8,
        }
    }
}

fn clear_text() {
    unsafe {
        ClearText();
    }
}

fn get_composed_text() -> Vec<Suggestion> {
    unsafe {
        let mut length: c_int = 0;
        let result = GetComposedText(&mut length);
        let mut suggestions = Vec::with_capacity(length as usize);

        for index in 0..length as usize {
            let candidate = (**result.add(index)).clone();
            let text = CStr::from_ptr(candidate.text)
                .to_string_lossy()
                .into_owned();
            let subtext = CStr::from_ptr(candidate.subtext)
                .to_string_lossy()
                .into_owned();
            let corresponding_count = candidate.corresponding_count;

            let suggestion = Suggestion {
                text,
                subtext,
                corresponding_count,
            };

            // check if suggestions have the same text
            if suggestions
                .iter()
                .any(|s: &Suggestion| s.text == suggestion.text)
            {
                continue;
            }
            suggestions.push(suggestion);
        }

        suggestions
    }
}

fn shrink_text(offset: i8) -> RawComposingText {
    unsafe {
        let offset = c_int::from(offset);
        let result = ShrinkText(offset);

        let text = CStr::from_ptr(&*result as *const c_char).to_str().unwrap();

        RawComposingText {
            text: text.to_string(),
            cursor: 0,
        }
    }
}

// いい感じ変換FFIラッパー関数
fn is_iikanji_keyword(input: &str) -> bool {
    unsafe {
        let input = CString::new(input).expect("CString::new failed");
        IsIikanjiKeyword(input.as_ptr())
    }
}

fn is_iikanji_enabled() -> bool {
    unsafe { IsIikanjiEnabled() }
}

fn request_iikanji(keyword: &str, context: &str) -> Option<String> {
    unsafe {
        let keyword = CString::new(keyword).expect("CString::new failed");
        let context = CString::new(context).expect("CString::new failed");
        
        let result = RequestIikanji(keyword.as_ptr(), context.as_ptr());
        
        if result.is_null() {
            return None;
        }
        
        let result_str = CStr::from_ptr(&*result as *const c_char)
            .to_str()
            .ok()?
            .to_string();
        
        Some(result_str)
    }
}

#[derive(Debug, Default)]
pub struct MyAzookeyService;

#[tonic::async_trait]
impl AzookeyService for MyAzookeyService {
    async fn append_text(
        &self,
        request: Request<AppendTextRequest>,
    ) -> Result<Response<AppendTextResponse>, Status> {
        let input = request.into_inner().text_to_append;
        let composing_text = add_text(&input);

        Ok(Response::new(AppendTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn remove_text(
        &self,
        _: Request<RemoveTextRequest>,
    ) -> Result<Response<RemoveTextResponse>, Status> {
        let composing_text = remove_text();

        Ok(Response::new(RemoveTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn move_cursor(
        &self,
        request: Request<MoveCursorRequest>,
    ) -> Result<Response<MoveCursorResponse>, Status> {
        let offset = request.into_inner().offset as i8;
        let composing_text = move_cursor(offset);

        Ok(Response::new(MoveCursorResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn clear_text(
        &self,
        _: Request<ClearTextRequest>,
    ) -> Result<Response<ClearTextResponse>, Status> {
        clear_text();
        Ok(Response::new(ClearTextResponse {}))
    }

    async fn shrink_text(
        &self,
        request: Request<ShrinkTextRequest>,
    ) -> Result<Response<ShrinkTextResponse>, Status> {
        let offset = request.into_inner().offset as i8;
        let composing_text = shrink_text(offset);

        Ok(Response::new(ShrinkTextResponse {
            composing_text: Some(ComposingText {
                hiragana: composing_text.text,
                suggestions: get_composed_text().to_vec(),
            }),
        }))
    }

    async fn set_context(
        &self,
        request: Request<shared::proto::SetContextRequest>,
    ) -> Result<Response<shared::proto::SetContextResponse>, Status> {
        let context = request.into_inner().context;
        let trimmed_context = context
            .split('\r')
            .filter(|s| !s.is_empty())
            .last()
            .unwrap_or_default();

        let context = CString::new(trimmed_context).expect("CString::new failed");

        unsafe { SetContext(context.as_ptr()) };
        Ok(Response::new(shared::proto::SetContextResponse {}))
    }

    async fn update_config(
        &self,
        _: Request<shared::proto::UpdateConfigRequest>,
    ) -> Result<Response<shared::proto::UpdateConfigResponse>, Status> {
        unsafe { LoadConfig() };
        Ok(Response::new(shared::proto::UpdateConfigResponse {}))
    }

    // いい感じ変換: キーワード判定
    async fn is_iikanji_keyword(
        &self,
        request: Request<IsIikanjiKeywordRequest>,
    ) -> Result<Response<IsIikanjiKeywordResponse>, Status> {
        let input = request.into_inner().input;
        let is_keyword = is_iikanji_keyword(&input);
        
        Ok(Response::new(IsIikanjiKeywordResponse { is_keyword }))
    }

    // いい感じ変換: 有効かどうか
    async fn is_iikanji_enabled(
        &self,
        _: Request<IsIikanjiEnabledRequest>,
    ) -> Result<Response<IsIikanjiEnabledResponse>, Status> {
        let enabled = is_iikanji_enabled();
        
        Ok(Response::new(IsIikanjiEnabledResponse { enabled }))
    }

    // いい感じ変換: 変換実行
    async fn request_iikanji(
        &self,
        request: Request<RequestIikanjiRequest>,
    ) -> Result<Response<RequestIikanjiResponse>, Status> {
        let req = request.into_inner();
        let keyword = req.keyword;
        let context = req.context;
        
        match request_iikanji(&keyword, &context) {
            Some(result) => Ok(Response::new(RequestIikanjiResponse {
                success: true,
                result,
                error: String::new(),
            })),
            None => Ok(Response::new(RequestIikanjiResponse {
                success: false,
                result: String::new(),
                error: "変換に失敗しました".to_string(),
            })),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AzookeyServer started");
    // get executable directory
    let current_exe = std::env::current_exe()?;
    let parent_dir = current_exe.parent().unwrap();
    initialize(parent_dir.to_str().unwrap());

    let service = MyAzookeyService::default();

    println!("AzookeyServer listening");

    Server::builder()
        .add_service(AzookeyServiceServer::new(service))
        .add_service(
            ReflectionBuilder::configure()
                .register_encoded_file_descriptor_set(shared::proto::FILE_DESCRIPTOR_SET)
                .build_v1()
                .unwrap(),
        )
        .serve_with_incoming(TonicNamedPipeServer::new("azookey_server"))
        .await?;

    Ok(())
}
