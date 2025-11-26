use anyhow::Result;
use hyper_util::rt::TokioIo;
use shared::proto::{
    azookey_service_client::AzookeyServiceClient, window_service_client::WindowServiceClient,
};
use std::{sync::Arc, time::Duration};
use tokio::{net::windows::named_pipe::ClientOptions, time};
use tonic::transport::Endpoint;
use tower::service_fn;
use windows::Win32::Foundation::ERROR_PIPE_BUSY;

// connect to kkc server
#[derive(Debug, Clone)]
pub struct IPCService {
    // kkc server client
    azookey_client: AzookeyServiceClient<tonic::transport::channel::Channel>,
    // candidate window server client (optional - may fail to connect)
    window_client: Option<WindowServiceClient<tonic::transport::channel::Channel>>,
    runtime: Arc<tokio::runtime::Runtime>,
}

#[derive(Debug, Clone, Default)]
pub struct Candidates {
    pub texts: Vec<String>,
    pub sub_texts: Vec<String>,
    pub hiragana: String,
    pub corresponding_count: Vec<i32>,
}

impl IPCService {
    pub fn new() -> Result<Self> {
        let runtime = tokio::runtime::Runtime::new()?;

        let server_channel = runtime.block_on(
            Endpoint::try_from("http://[::]:50051")?.connect_with_connector(service_fn(
                |_| async {
                    let client = loop {
                        match ClientOptions::new().open(r"\\.\pipe\azookey_server") {
                            Ok(client) => break client,
                            Err(e) if e.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => (),
                            Err(e) => return Err(e),
                        }

                        time::sleep(Duration::from_millis(50)).await;
                    };

                    Ok::<_, std::io::Error>(TokioIo::new(client))
                },
            )),
        )?;

        // UI接続は完全にスキップ（開発ビルドではUIが起動できないため）
        // TODO: 署名付きビルドでUIを有効にする
        let window_client: Option<WindowServiceClient<tonic::transport::channel::Channel>> = None;
        tracing::info!("UI connection skipped (development build)");

        let azookey_client = AzookeyServiceClient::new(server_channel);
        tracing::debug!("Connected to server: {:?}", azookey_client);

        Ok(Self {
            azookey_client,
            window_client,
            runtime: Arc::new(runtime),
        })
    }
}

// implement methods to interact with kkc server
impl IPCService {
    #[tracing::instrument]
    pub fn append_text(&mut self, text: String) -> anyhow::Result<Candidates> {
        let request = tonic::Request::new(shared::proto::AppendTextRequest {
            text_to_append: text,
        });

        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.append_text(request))?;
        let composing_text = response.into_inner().composing_text;

        let candidates = if let Some(composing_text) = composing_text {
            Candidates {
                texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.text.clone())
                    .collect(),
                sub_texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.subtext.clone())
                    .collect(),
                hiragana: composing_text.hiragana,
                corresponding_count: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.corresponding_count)
                    .collect(),
            }
        } else {
            anyhow::bail!("composing_text is None");
        };

        Ok(candidates)
    }

    #[tracing::instrument]
    pub fn remove_text(&mut self) -> anyhow::Result<Candidates> {
        let request = tonic::Request::new(shared::proto::RemoveTextRequest {});
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.remove_text(request))?;
        let composing_text = response.into_inner().composing_text;

        let candidates = if let Some(composing_text) = composing_text {
            Candidates {
                texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.text.clone())
                    .collect(),
                sub_texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.subtext.clone())
                    .collect(),
                hiragana: composing_text.hiragana,
                corresponding_count: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.corresponding_count)
                    .collect(),
            }
        } else {
            anyhow::bail!("composing_text is None");
        };

        Ok(candidates)
    }

    #[tracing::instrument]
    pub fn clear_text(&mut self) -> anyhow::Result<()> {
        let request = tonic::Request::new(shared::proto::ClearTextRequest {});
        let _response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.clear_text(request))?;

        Ok(())
    }

    #[tracing::instrument]
    pub fn shrink_text(&mut self, offset: i32) -> anyhow::Result<Candidates> {
        let request = tonic::Request::new(shared::proto::ShrinkTextRequest { offset });
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.shrink_text(request))?;
        let composing_text = response.into_inner().composing_text;

        let candidates = if let Some(composing_text) = composing_text {
            Candidates {
                texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.text.clone())
                    .collect(),
                sub_texts: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.subtext.clone())
                    .collect(),
                hiragana: composing_text.hiragana,
                corresponding_count: composing_text
                    .suggestions
                    .iter()
                    .map(|s| s.corresponding_count)
                    .collect(),
            }
        } else {
            anyhow::bail!("composing_text is None");
        };

        Ok(candidates)
    }

    /// ライブ変換: 全候補取得（変換キー押下時に呼ぶ）
    #[tracing::instrument]
    pub fn get_all_candidates(&mut self, hiragana: &str) -> anyhow::Result<Candidates> {
        let request = tonic::Request::new(shared::proto::GetAllCandidatesRequest {});
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.get_all_candidates(request))?;
        let suggestions = response.into_inner().suggestions;

        let candidates = Candidates {
            texts: suggestions.iter().map(|s| s.text.clone()).collect(),
            sub_texts: suggestions.iter().map(|s| s.subtext.clone()).collect(),
            hiragana: hiragana.to_string(),
            corresponding_count: suggestions.iter().map(|s| s.corresponding_count).collect(),
        };

        Ok(candidates)
    }

    pub fn set_context(&mut self, context: String) -> anyhow::Result<()> {
        let request = tonic::Request::new(shared::proto::SetContextRequest { context });
        let _response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.set_context(request))?;

        Ok(())
    }
}

// implement methods to interact with candidate window server
impl IPCService {
    #[tracing::instrument]
    pub fn show_window(&mut self) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::EmptyResponse {});
            self.runtime.clone().block_on(client.show_window(request))?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn hide_window(&mut self) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::EmptyResponse {});
            self.runtime.clone().block_on(client.hide_window(request))?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn set_window_position(
        &mut self,
        top: i32,
        left: i32,
        bottom: i32,
        right: i32,
    ) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::SetPositionRequest {
                position: Some(shared::proto::WindowPosition {
                    top,
                    left,
                    bottom,
                    right,
                }),
            });
            self.runtime
                .clone()
                .block_on(client.set_window_position(request))?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn set_candidates(&mut self, candidates: Vec<String>) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::SetCandidateRequest { candidates });
            self.runtime.clone().block_on(client.set_candidate(request))?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn set_selection(&mut self, index: i32) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::SetSelectionRequest { index });
            self.runtime.clone().block_on(client.set_selection(request))?;
        }
        Ok(())
    }

    #[tracing::instrument]
    pub fn set_input_mode(&mut self, mode: &str) -> anyhow::Result<()> {
        if let Some(ref mut client) = self.window_client {
            let request = tonic::Request::new(shared::proto::SetInputModeRequest {
                mode: mode.to_string(),
            });
            self.runtime.clone().block_on(client.set_input_mode(request))?;
        }
        Ok(())
    }
}

// いい感じ変換関連メソッド
impl IPCService {
    /// いい感じ変換が有効かどうかを確認
    #[tracing::instrument]
    pub fn is_iikanji_enabled(&mut self) -> anyhow::Result<bool> {
        let request = tonic::Request::new(shared::proto::IsIikanjiEnabledRequest {});
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.is_iikanji_enabled(request))?;

        Ok(response.into_inner().enabled)
    }

    /// 入力がいい感じ変換キーワードかどうかを判定
    #[tracing::instrument]
    pub fn is_iikanji_keyword(&mut self, input: &str) -> anyhow::Result<bool> {
        let request = tonic::Request::new(shared::proto::IsIikanjiKeywordRequest {
            input: input.to_string(),
        });
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.is_iikanji_keyword(request))?;

        Ok(response.into_inner().is_keyword)
    }

    /// いい感じ変換を実行
    #[tracing::instrument]
    pub fn request_iikanji(
        &mut self,
        keyword: &str,
        context: &str,
    ) -> anyhow::Result<Option<String>> {
        let request = tonic::Request::new(shared::proto::RequestIikanjiRequest {
            keyword: keyword.to_string(),
            context: context.to_string(),
        });
        let response = self
            .runtime
            .clone()
            .block_on(self.azookey_client.request_iikanji(request))?;

        let inner = response.into_inner();
        if inner.success {
            Ok(Some(inner.result))
        } else {
            tracing::warn!("Iikanji request failed: {}", inner.error);
            Ok(None)
        }
    }
}
