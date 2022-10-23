use std::io;
use lettre::{Message, SmtpTransport, Transport};
use std::io::{stdout, Write};

pub async fn send_email() -> io::Result<()> {
    let message = Message::builder()
        .from("NoBody <nobody@domain.tld>".parse().unwrap())
        .reply_to("Yuin <yuin@domain.tld>".parse().unwrap())
        .to("Hei <hei@domain.tld>".parse().unwrap())
        .subject("Happy new year")
        .body(String::from("Be happy!"))
        .unwrap();

    let mailer = SmtpTransport::builder_dangerous("localhost")
        .port(1025)
        .build();

    mailer.send(&message).expect("message should be sent");
//
// // Send the email
// //     let result = match mailer.send(&message) {
// //         Ok(_) => "Email sent successfully!",
// //         Err(e) => {
// //             println!("Could not send email: {:?}", e);
// //             "Could not send email"
// //         },
// //     };
//
    stdout()
        .write("message sent".as_bytes())
        .map(|_| ())
}
