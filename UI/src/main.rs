use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Button, Box, Orientation};

fn main() {
    let application = Application::builder()
        .application_id("com.example.FirstGtkApp")
        .build();

    application.connect_activate(|app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Placeholder")
            .default_width(350)
            .default_height(70)
            .build();

        let cont=Box::new(Orientation::Vertical,10);
        cont.set_margin_top(500);
        cont.set_margin_start(800);
        cont.set_margin_end(800);

        let button_auth = Button::with_label("Authentificate");
        let button_reg = Button::with_label("Register");

        button_reg.connect_clicked(|_| {
            eprintln!("request_reg");
        });
        button_auth.connect_clicked(|_| {
            eprintln!("request_auth");
        });

        cont.add(&button_auth);
        cont.add(&button_reg);
        window.add(&cont);
        window.show_all();
    });

    application.run();
}
