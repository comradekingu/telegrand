use grammers_client::{Client, Config};
use grammers_session::FileSession;
use gtk::glib;
use std::sync::mpsc;
use tokio::runtime;

use crate::config;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub enum MessageGTK {
    AccountNotAuthorized,
    NeedConfirmationCode,
    SuccessfullySignedIn,
}

pub enum MessageTG {
    SendPhoneNumber(String),
    SendConfirmationCode(String),
}

pub fn spawn(gtk_sender: glib::Sender<MessageGTK>, tg_receiver: mpsc::Receiver<MessageTG>) {
    std::thread::spawn(move || {
        let _ = runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(start(gtk_sender, tg_receiver));
    });
}

async fn start(gtk_sender: glib::Sender<MessageGTK>, tg_receiver: mpsc::Receiver<MessageTG>) -> Result<()> {
    let api_id = config::TG_API_ID.to_owned();
    let api_hash = config::TG_API_HASH.to_owned();

    let mut client = Client::connect(Config {
        session: FileSession::load_or_create("telegrand.session")?,
        api_id,
        api_hash: api_hash.clone(),
        params: Default::default(),
    })
    .await?;

    if !client.is_authorized().await? {
        gtk_sender.send(MessageGTK::AccountNotAuthorized).unwrap();

        let mut message = tg_receiver.recv().unwrap();
        let mut signed_in = false;

        while !signed_in {
            if let MessageTG::SendPhoneNumber(ref number) = message {
                match client.request_login_code(&number, api_id, &api_hash).await {
                    Ok(token) => {
                        gtk_sender.send(MessageGTK::NeedConfirmationCode).unwrap();
                        message = tg_receiver.recv().unwrap();

                        if let MessageTG::SendConfirmationCode(ref code) = message {
                            match client.sign_in(&token, &code).await {
                                Ok(_) => {
                                    gtk_sender.send(MessageGTK::SuccessfullySignedIn).unwrap();
                                    signed_in = true;
                                },
                                Err(e) => panic!(e),
                            }
                        }
                    },
                    Err(e) => panic!(e),
                };
            } else {
                message = tg_receiver.recv().unwrap();
            }
        }

        // TODO: sign out when closing the app if this fails.
        client.session().save().unwrap();
    }

    Ok(())
}
