use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use tdlib::types::Error as TdError;
use tdlib::{enums, functions};

use crate::session::chat::BoxedMessageContent;
use crate::session::Chat;
use crate::Session;

mod imp {
    use super::*;
    use glib::WeakRef;
    use once_cell::sync::Lazy;
    use once_cell::unsync::OnceCell;
    use std::cell::Cell;

    #[derive(Debug, Default)]
    pub(crate) struct SponsoredMessage {
        pub(super) message_id: Cell<i64>,
        pub(super) content: OnceCell<BoxedMessageContent>,
        pub(super) sponsor_chat: WeakRef<Chat>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SponsoredMessage {
        const NAME: &'static str = "ChatSponsoredMessage";
        type Type = super::SponsoredMessage;
    }

    impl ObjectImpl for SponsoredMessage {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> = Lazy::new(|| {
                vec![
                    glib::ParamSpecInt64::new(
                        "message-id",
                        "Message Id",
                        "The id of this message",
                        std::i64::MIN,
                        std::i64::MAX,
                        0,
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecBoxed::new(
                        "content",
                        "Content",
                        "The content of this message",
                        BoxedMessageContent::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                    glib::ParamSpecObject::new(
                        "sponsor-chat",
                        "Sponsor Chat",
                        "The chat relative to this sponsored message",
                        Chat::static_type(),
                        glib::ParamFlags::READWRITE | glib::ParamFlags::CONSTRUCT_ONLY,
                    ),
                ]
            });
            PROPERTIES.as_ref()
        }

        fn set_property(
            &self,
            _obj: &Self::Type,
            _id: usize,
            value: &glib::Value,
            pspec: &glib::ParamSpec,
        ) {
            match pspec.name() {
                "message-id" => self.message_id.set(value.get().unwrap()),
                "content" => self.content.set(value.get().unwrap()).unwrap(),
                "sponsor-chat" => self.sponsor_chat.set(Some(&value.get().unwrap())),
                _ => unimplemented!(),
            }
        }

        fn property(&self, obj: &Self::Type, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "message-id" => obj.message_id().to_value(),
                "content" => obj.content().to_value(),
                "sponsor-chat" => obj.sponsor_chat().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub(crate) struct SponsoredMessage(ObjectSubclass<imp::SponsoredMessage>);
}

impl SponsoredMessage {
    pub(crate) async fn request(chat_id: i64, session: &Session) -> Result<Self, TdError> {
        let enums::SponsoredMessage::SponsoredMessage(sponsored_message) =
            functions::get_chat_sponsored_message(chat_id, session.client_id()).await?;

        let content = BoxedMessageContent(sponsored_message.content);
        let sponsor_chat = session.chat_list().get(sponsored_message.sponsor_chat_id);

        Ok(glib::Object::new(&[
            ("message-id", &sponsored_message.message_id),
            ("content", &content),
            ("sponsor-chat", &sponsor_chat),
        ])
        .expect("Failed to create ChatSponsoredMessage"))
    }

    pub(crate) fn message_id(&self) -> i64 {
        self.imp().message_id.get()
    }

    pub(crate) fn content(&self) -> &BoxedMessageContent {
        self.imp().content.get().unwrap()
    }

    pub(crate) fn sponsor_chat(&self) -> Chat {
        self.imp().sponsor_chat.upgrade().unwrap()
    }
}
