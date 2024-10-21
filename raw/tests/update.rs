use std::fs::File;
use std::io::prelude::*;

use telegram_bot_raw::types::message::{Message, MessageKind};
use telegram_bot_raw::types::update::{Update, UpdateKind};

macro_rules! make_test {
    ($asset: ident, $test: expr) => {
        #[test]
        fn $asset() {
            let data = {
                let filename = format!("tests/update_assets/{}.json", stringify!($asset));
                let mut data = Vec::new();
                let mut file = File::open(filename).unwrap();
                file.read_to_end(&mut data).unwrap();
                data
            };
            let update = serde_json::from_slice::<Update>(&data).unwrap();
            $test(update)
        }
    };
}

make_test!(migrate_from_chat_id, |update: Update| {
    assert!(matches!(
        update.kind,
        UpdateKind::Message(Message {
            kind: MessageKind::MigrateFromChatId { .. },
            ..
        })
    ));
});

make_test!(migrate_to_chat_id, |update: Update| {
    assert!(matches!(
        update.kind,
        UpdateKind::Message(Message {
            kind: MessageKind::MigrateToChatId { .. },
            ..
        })
    ));
});

make_test!(inline_query, |update: Update| {
    assert!(matches!(update.kind, UpdateKind::InlineQuery(_)));
});

make_test!(regression_test_208, |update: Update| {
    assert!(matches!(update.kind, UpdateKind::CallbackQuery(_)));
});
