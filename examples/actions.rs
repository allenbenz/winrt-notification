extern crate winrt_notification;

use winrt_notification::{
    Action,
    Toast,
    ToastWithHandlers,
};

fn main() {
    let toast = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("toast with input")
        .action(Action {
            content: "Hey".to_string(),
            arguments: "hey".to_string(),
            place_to_context_menu: false,
        })
        .action(Action::from_content("Hey2"))
        .action(Action {
            content: "Context Action 1".to_string(),
            arguments: "context 1".to_string(),
            place_to_context_menu: true,
        });
    ToastWithHandlers::new(toast)
        .on_activate(|args| {
            println!("arguments: {}", args.get_arguments().unwrap()?);
            // exit the waiting loop
            std::process::exit(0);
        })
        .show()
        .expect("unable to send notification");
    // wait for activation
    loop {}
}
