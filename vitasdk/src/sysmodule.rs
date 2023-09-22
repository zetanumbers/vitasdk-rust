use core::mem;

use vitasdk_sys::{sceSysmoduleLoadModule, sceSysmoduleUnloadModule, SceSysmoduleModuleId};

use crate::error::{sce_result_unit_from_code, SceResult};

pub struct Module {
    id: ModuleId,
}

impl Module {
    pub fn load(id: ModuleId) -> SceResult<Self> {
        sce_result_unit_from_code(unsafe { sceSysmoduleLoadModule(id.0) })?;
        Ok(Module { id })
    }

    /// Does the same thing as drop, but you could handle the error case.
    pub fn unload(self) -> SceResult<()> {
        mem::ManuallyDrop::new(self).unload_()
    }

    fn unload_(&mut self) -> SceResult<()> {
        sce_result_unit_from_code(unsafe { sceSysmoduleUnloadModule(self.id.0) })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let _ = self.unload_();
    }
}

#[derive(Clone, Copy)]
pub struct ModuleId(SceSysmoduleModuleId);

impl ModuleId {
    pub const NET: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NET);
    pub const HTTP: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_HTTP);
    pub const SSL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SSL);
    pub const HTTPS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_HTTPS);
    pub const PERF: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PERF);
    pub const FIBER: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_FIBER);
    pub const ULT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_ULT);
    pub const DBG: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_DBG);
    pub const RAZOR_CAPTURE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_RAZOR_CAPTURE);
    pub const RAZOR_HUD: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_RAZOR_HUD);
    pub const NGS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NGS);
    pub const SULPHA: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SULPHA);
    pub const SAS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SAS);
    pub const PGF: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PGF);
    pub const APPUTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_APPUTIL);
    pub const FIOS2: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_FIOS2);
    pub const IME: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_IME);
    pub const NP_BASIC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_BASIC);
    pub const SYSTEM_GESTURE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SYSTEM_GESTURE);
    pub const LOCATION: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_LOCATION);
    pub const NP: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP);
    pub const PHOTO_EXPORT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PHOTO_EXPORT);
    pub const XML: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_XML);
    pub const NP_COMMERCE2: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_COMMERCE2);
    pub const NP_UTILITY: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_UTILITY);
    pub const VOICE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_VOICE);
    pub const VOICEQOS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_VOICEQOS);
    pub const NP_MATCHING2: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_MATCHING2);
    pub const SCREEN_SHOT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SCREEN_SHOT);
    pub const NP_SCORE_RANKING: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_SCORE_RANKING);
    pub const SQLITE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SQLITE);
    pub const TRIGGER_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_TRIGGER_UTIL);
    pub const RUDP: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_RUDP);
    pub const CODECENGINE_PERF: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_CODECENGINE_PERF);
    pub const LIVEAREA: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_LIVEAREA);
    pub const NP_ACTIVITY: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_ACTIVITY);
    pub const NP_TROPHY: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_TROPHY);
    pub const NP_MESSAGE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_MESSAGE);
    pub const SHUTTER_SOUND: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SHUTTER_SOUND);
    pub const CLIPBOARD: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_CLIPBOARD);
    pub const NP_PARTY: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_PARTY);
    pub const NET_ADHOC_MATCHING: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NET_ADHOC_MATCHING);
    pub const NEAR_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NEAR_UTIL);
    pub const NP_TUS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_TUS);
    pub const MP4: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MP4);
    pub const AACENC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_AACENC);
    pub const HANDWRITING: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_HANDWRITING);
    pub const ATRAC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_ATRAC);
    pub const NP_SNS_FACEBOOK: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_SNS_FACEBOOK);
    pub const VIDEO_EXPORT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_VIDEO_EXPORT);
    pub const NOTIFICATION_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NOTIFICATION_UTIL);
    pub const BG_APP_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_BG_APP_UTIL);
    pub const INCOMING_DIALOG: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_INCOMING_DIALOG);
    pub const IPMI: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_IPMI);
    pub const AUDIOCODEC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_AUDIOCODEC);
    pub const FACE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_FACE);
    pub const SMART: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SMART);
    pub const MARLIN: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MARLIN);
    pub const MARLIN_DOWNLOADER: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MARLIN_DOWNLOADER);
    pub const MARLIN_APP_LIB: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MARLIN_APP_LIB);
    pub const TELEPHONY_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_TELEPHONY_UTIL);
    pub const SHACCCG: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_SHACCCG);
    pub const MONO_BRIDGE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MONO_BRIDGE);
    pub const MONO: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MONO);
    pub const PSM: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PSM);
    pub const PSM_DEVAGENT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PSM_DEVAGENT);
    pub const PSPNET_ADHOC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_PSPNET_ADHOC);
    pub const DTCP_IP: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_DTCP_IP);
    pub const VIDEO_SEARCH_EMPR: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_VIDEO_SEARCH_EMPR);
    pub const NP_SIGNALING: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_SIGNALING);
    pub const BEISOBMF: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_BEISOBMF);
    pub const BEMP2SYS: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_BEMP2SYS);
    pub const MUSIC_EXPORT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MUSIC_EXPORT);
    pub const NEAR_DIALOG_UTIL: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NEAR_DIALOG_UTIL);
    pub const LOCATION_EXTENSION: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_LOCATION_EXTENSION);
    pub const AVPLAYER: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_AVPLAYER);
    pub const GAME_UPDATE: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_GAME_UPDATE);
    pub const MAIL_API: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MAIL_API);
    pub const TELEPORT_CLIENT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_TELEPORT_CLIENT);
    pub const TELEPORT_SERVER: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_TELEPORT_SERVER);
    pub const MP4_RECORDER: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_MP4_RECORDER);
    pub const APPUTIL_EXT: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_APPUTIL_EXT);
    pub const NP_WEBAPI: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_NP_WEBAPI);
    pub const AVCDEC: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_AVCDEC);
    pub const JSON: Self = ModuleId(vitasdk_sys::SCE_SYSMODULE_JSON);
}
