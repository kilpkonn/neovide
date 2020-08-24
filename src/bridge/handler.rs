use async_trait::async_trait;
use log::{error, trace};
use nvim_rs::{compat::tokio::Compat, Handler, Neovim};
use rmpv::Value;
use tokio::process::ChildStdin;
use tokio::task;

use super::events::handle_redraw_event_group;
use crate::settings::SETTINGS;

#[cfg(windows)]
use crate::settings::windows_registry::{
    register_rightclick_directory, register_rightclick_file, unregister_rightclick,
};

#[derive(Clone)]
pub struct NeovimHandler();

#[async_trait]
impl Handler for NeovimHandler {
    type Writer = Compat<ChildStdin>;

    async fn handle_notify(
        &self,
        event_name: String,
        arguments: Vec<Value>,
        _neovim: Neovim<Compat<ChildStdin>>,
    ) {
        trace!("Neovim notification: {:?}", &event_name);
        task::spawn_blocking(move || match event_name.as_ref() {
            "redraw" => {
                handle_redraw_event_group(arguments);
            }
            "setting_changed" => {
                SETTINGS.handle_changed_notification(arguments);
            }
            "neovide.register_right_click" => {
                if cfg!(windows) {
                    if unregister_rightclick() {
                        error!("Setup of Windows Registry failed during unregister. Try running as Admin?");
                    }
                    if !register_rightclick_directory() {
                        error!("Setup of Windows Registry failed during directory registration. Try running as Admin?");
                    }
                    if !register_rightclick_file() {
                        error!("Setup of Windows Registry failed during file registration. Try running as Admin?");
                    }
                }
            }
            "neovide.unregister_right_click" => {
                if cfg!(windows) && !unregister_rightclick() {
                    error!("Removal of Windows Registry failed, probably no Admin");
                }
            }
            _ => {}
        })
        .await
        .ok();
    }
}
