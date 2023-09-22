use std::{ffi::CStr, mem};

use vitasdk_sys::{
    sceHttpCreateConnectionWithURL, sceHttpCreateRequest, sceHttpCreateRequestWithURL,
    sceHttpCreateTemplate, sceHttpDeleteConnection, sceHttpDeleteRequest, sceHttpDeleteTemplate,
    sceHttpInit, sceHttpTerm, SceHttpMethods,
};

use crate::{
    error::{sce_result_uid_from_code, sce_result_unit_from_code, SceResult},
    module::{Module, ModuleId},
    types::Uid,
};

pub struct GlobalState {
    _module: Module,
}

impl GlobalState {
    pub fn new() -> SceResult<Self> {
        let module = Module::load(ModuleId::HTTPS)?;
        sce_result_unit_from_code(unsafe { sceHttpInit(0x400000) })?;
        Ok(GlobalState { _module: module })
    }
}

impl Drop for GlobalState {
    fn drop(&mut self) {
        let _ = unsafe { sceHttpTerm() };
    }
}

#[derive(Debug)]
pub struct Template {
    uid: Uid,
}

impl Template {
    pub fn new(user_agent: &CStr, http_ver: i32, auto_proxy_conf: i32) -> SceResult<Self> {
        Ok(Template {
            uid: sce_result_uid_from_code(unsafe {
                sceHttpCreateTemplate(user_agent.as_ptr(), http_ver, auto_proxy_conf)
            })?,
        })
    }

    pub fn create_connection_with_url(
        &self,
        url: &CStr,
        keep_alive: KeepAlive,
    ) -> SceResult<Connection> {
        Ok(Connection {
            uid: sce_result_uid_from_code(unsafe {
                sceHttpCreateConnectionWithURL(
                    self.uid.get(),
                    url.as_ptr(),
                    keep_alive.as_bool() as i32,
                )
            })?,
        })
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn delete(self) -> SceResult<()> {
        mem::ManuallyDrop::new(self).delete_()
    }

    fn delete_(&mut self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { sceHttpDeleteTemplate(self.uid.get()) })
    }
}

impl Drop for Template {
    fn drop(&mut self) {
        let _ = self.delete_();
    }
}

#[derive(Debug)]
pub struct Connection {
    uid: Uid,
}

impl Connection {
    pub fn create_request(
        &self,
        method: Method,
        path: &CStr,
        content_length: u64,
    ) -> SceResult<Request> {
        Ok(Request {
            uid: sce_result_uid_from_code(unsafe {
                sceHttpCreateRequest(
                    self.uid.get(),
                    method.0 as i32,
                    path.as_ptr(),
                    content_length,
                )
            })?,
        })
    }

    pub fn create_request_with_url(
        &self,
        method: Method,
        url: &CStr,
        content_length: u64,
    ) -> SceResult<Request> {
        Ok(Request {
            uid: sce_result_uid_from_code(unsafe {
                sceHttpCreateRequestWithURL(
                    self.uid.get(),
                    method.0 as i32,
                    url.as_ptr(),
                    content_length,
                )
            })?,
        })
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn delete(self) -> SceResult<()> {
        mem::ManuallyDrop::new(self).delete_()
    }

    fn delete_(&mut self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { sceHttpDeleteConnection(self.uid.get()) })
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.delete_();
    }
}

pub struct Request {
    uid: Uid,
}

impl Request {
    pub fn send(&self, post_data: &[u8]) -> SceResult<()> {
        todo!()
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn delete(self) -> SceResult<()> {
        mem::ManuallyDrop::new(self).delete_()
    }

    fn delete_(&mut self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { sceHttpDeleteRequest(self.uid.get()) })
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        let _ = self.delete_();
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum KeepAlive {
    Enable,
    #[default]
    Disable,
}

impl KeepAlive {
    pub const fn as_bool(self) -> bool {
        match self {
            KeepAlive::Enable => true,
            KeepAlive::Disable => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Method(SceHttpMethods::Type);

impl Method {
    pub const GET: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_GET);
    pub const POST: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_POST);
    pub const HEAD: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_HEAD);
    pub const OPTIONS: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_OPTIONS);
    pub const PUT: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_PUT);
    pub const DELETE: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_DELETE);
    pub const TRACE: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_TRACE);
    pub const CONNECT: Self = Method(SceHttpMethods::SCE_HTTP_METHOD_CONNECT);
}
