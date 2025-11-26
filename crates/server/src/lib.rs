use async_stream::stream;
use futures_core::stream::Stream;
use std::{ffi::c_void, pin::Pin, ptr::addr_of_mut};
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    net::windows::named_pipe::{NamedPipeServer, ServerOptions},
};
use tonic::transport::server::Connected;
use windows::{
    core::w,
    Win32::Security::{
        Authorization::{ConvertStringSecurityDescriptorToSecurityDescriptorW, SDDL_REVISION},
        PSECURITY_DESCRIPTOR, SECURITY_ATTRIBUTES,
    },
};

#[allow(dead_code)]
struct UnsafeSecurityAttributes(SECURITY_ATTRIBUTES);

unsafe impl Send for UnsafeSecurityAttributes {}
unsafe impl Sync for UnsafeSecurityAttributes {}

pub struct TonicNamedPipeServer {
    inner: NamedPipeServer,
}

impl Connected for TonicNamedPipeServer {
    type ConnectInfo = ();

    fn connect_info(&self) -> Self::ConnectInfo {
        ()
    }
}

impl AsyncRead for TonicNamedPipeServer {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for TonicNamedPipeServer {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

impl TonicNamedPipeServer {
    pub fn new(path: &str) -> impl Stream<Item = io::Result<TonicNamedPipeServer>> {
        // set security attributes to allow ipc from sandboxed processes
        // see https://nathancorvussolis.blogspot.com/2018/05/windows-ime-security.html

        let name = format!("\\\\.\\pipe\\{}", path);

        let mut security_descriptor = PSECURITY_DESCRIPTOR::default();

        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                w!("D:(A;;GA;;;AC)(A;;GA;;;RC)(A;;GA;;;SY)(A;;GA;;;BA)(A;;GA;;;BU)S:(ML;;NW;;;LW)"),
                SDDL_REVISION,
                &mut security_descriptor,
                None,
            ).expect("Failed to create security descriptor");

            let mut security_attributes = UnsafeSecurityAttributes(SECURITY_ATTRIBUTES {
                nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
                lpSecurityDescriptor: security_descriptor.0,
                bInheritHandle: false.into(),
            });

            stream! {
                let server_result = ServerOptions::new()
                    .first_pipe_instance(true)
                    .create_with_security_attributes_raw(
                        &name,
                        addr_of_mut!(security_attributes) as *mut c_void
                    );

                let mut server = match server_result {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[Pipe] Error creating pipe: {:?}", e);
                        yield Err(e);
                        return;
                    }
                };

                loop {
                    match server.connect().await {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("[Pipe] Connection error: {:?}", e);
                            yield Err(e);
                            return;
                        }
                    }

                    let client = TonicNamedPipeServer {
                        inner: server,
                    };

                    yield Ok(client);

                    let next_server_result = ServerOptions::new()
                        .create_with_security_attributes_raw(
                            &name,
                            addr_of_mut!(security_attributes) as *mut c_void
                        );

                    server = match next_server_result {
                        Ok(s) => s,
                        Err(e) => {
                            eprintln!("[Pipe] Error creating next pipe: {:?}", e);
                            yield Err(e);
                            return;
                        }
                    };
                }
            }
        }
    }
}
