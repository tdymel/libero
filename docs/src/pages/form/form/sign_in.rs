use dioxus::prelude::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    task::{Poll, Waker},
    time::Duration,
};

// demo-code: start
use libero::components::{
    Button, ByAddress, Fields, Form, FormErrors, PasswordField, RequiredValue, Submit, SummaryMode,
    Text, TextField, use_form,
};
use std::convert::Infallible;

/// A secret: no `Clone` and no `PartialEq`, so it is never copied by accident.
struct Password(String);

#[derive(Clone, PartialEq, Default, Fields)]
struct Login {
    email: String,
    // `ByAddress` makes the secret fit for a form value.
    password: Option<ByAddress<Password>>,
}

fn password_text(password: &ByAddress<Password>) -> String {
    password.0.clone()
}

fn password(text: String) -> Result<ByAddress<Password>, Infallible> {
    Ok(ByAddress::new(Password(text)))
}

struct Credentials {
    email: String,
    password: ByAddress<Password>,
}

fn credentials(login: &Login) -> Result<Credentials, FormErrors> {
    let password = login.password.clone().required(Login::FIELDS.password())?;
    Ok(Credentials { email: login.email.clone(), password })
}

#[component]
pub fn SignInForm() -> Element {
    let login = use_store(Login::default);
    let form = use_form();
    let mut signed_in = use_signal(String::new);

    rsx! {
        Form {
            sx: libero::sx::sx().width("100%").max_width("320px"),
            form,
            value: login,
            // Each field shows its own error, and focus goes to the first one.
            summary: SummaryMode::Hidden,
            // While the request runs, the form ignores another submit.
            onsubmit: Submit::parse_async(credentials, move |sent: Credentials| async move {
                if !sign_in(&sent).await {
                    // No field is at fault, so the error shows above the fields.
                    return Err(FormErrors::new().form("Wrong email or password."));
                }
                signed_in.set(sent.email);
                Ok(())
            }),
            TextField { label: "Email", r#type: "email", autocomplete: "username", name: Login::FIELDS.email(), required: true }
            PasswordField {
                label: "Password",
                autocomplete: "current-password",
                name: Login::FIELDS.password().map(password_text, password),
                required: true,
            }
            Button { r#type: "submit", loading: form.is_submitting(), "Sign in" }
            Text { role: "status",
                if !signed_in().is_empty() { "Signed in as {signed_in}." }
            }
        }
    }
}
// demo-code: end

/// Stands in for a server: a second's wait, then only `secret` passes.
async fn sign_in(sent: &Credentials) -> bool {
    delay(Duration::from_secs(1)).await;
    sent.password.0 == "secret"
}

pub(super) async fn delay(duration: Duration) {
    let Some(timer) = libero::platform::timer() else {
        return;
    };
    let done = Rc::new(Cell::new(false));
    let waker = Rc::new(RefCell::new(None::<Waker>));
    let (fired, wake) = (done.clone(), waker.clone());
    let _pending = timer.after(
        duration,
        Box::new(move || {
            fired.set(true);
            if let Some(waker) = wake.borrow_mut().take() {
                waker.wake();
            }
        }),
    );
    std::future::poll_fn(|context| match done.get() {
        true => Poll::Ready(()),
        false => {
            *waker.borrow_mut() = Some(context.waker().clone());
            Poll::Pending
        }
    })
    .await
}
