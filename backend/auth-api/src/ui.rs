use maud::html;

pub fn layout(title: &str, content: String) -> String {
    let markup = html! {
        (maud::PreEscaped("<!DOCTYPE html>"))
        html lang="ru" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (title) }
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/@picocss/pico@1/css/pico.min.css";
                script src="https://unpkg.com/htmx.org@1.9.10" {}
            }
            body {
                nav class="container" {
                    ul {
                        li { a href="/login" strong { "e-Market" } }
                    }
                    ul {
                        li { a href="/login" { "Log in" } }
                        li { a href="/register" { "Registration" } }
                    }
                }
                main class="container" {
                    div id="ui-content" {
                        (maud::PreEscaped(&content))
                    }
                }
            }
        }
    };
    markup.into_string()
}

pub fn login_form() -> String {
    let form = html! {
        article {
            header { "User authorization" }
            form hx-post="/api/users/login" hx-target="#error-container" hx-swap="innerHTML" {
                label for="login" { "Email" }
                input type="email" id="login" name="login" placeholder="ivan@example.com" required;

                label for="password" { "Password" }
                input type="password" id="password" name="password" required;

                button type="submit" { "Log in" }
            }
            p {
                a href="/forgot-password" { "Forgot password?" }
            }
            div id="error-container" {}
        }
    };
    form.into_string()
}

pub fn register_form() -> String {
    let form = html! {
        article id="register-card" {
            header { "Create your account" }

            div id="registration-container" {

                fieldset {
                    legend { "Register as:" }
                    label for="role_customer" {
                        input type="radio" id="role_customer" name="ui-role" value="customer" checked
                            hx-get="/register/customer-fragment" hx-target="#form-body" hx-swap="innerHTML";
                        "Customer"
                    }
                    label for="role_vendor" {
                        input type="radio" id="role_vendor" name="ui-role" value="vendor"
                            hx-get="/register/vendor-fragment" hx-target="#form-body" hx-swap="innerHTML";
                        "Vendor"
                    }
                }

                div id="form-body" {
                    (maud::PreEscaped(&confirm_button("customer")))
                }
            }
            div id="error-container" {}
        }
    };
    form.into_string()
}

pub fn confirm_button(role: &str) -> String {
    let markup = html! {
        form hx-post=(format!("/api/users/{}s/register", role)) hx-target="#error-container" hx-swap="innerHTML" {
            (shared_fields())
            button type="submit" { (format!("Confirm Registration as {}", role)) }
        }
    };
    markup.into_string()
}

fn shared_fields() -> maud::Markup {
    html! {
        label for="first_name" { "Name" }
        input type="text" id="first_name" name="first_name" placeholder="Ivan" required;

        label for="second_name" { "Surname" }
                input type="text" id="second_name" name="second_name" placeholder="Ivanov" required;

        label for="email" { "Email" }
        input type="email" id="email" name="email" placeholder="ivan@example.com" required;

        label for="password" { "Password" }
        input type="password" id="password" name="password" required;
    }
}

pub fn html_error_fragment(message: &str) -> String {
    let fragment = html! {
        div style="padding: 1rem; margin-top: 1rem; background-color: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; border-radius: 4px;" {
            strong { "Error: " } (message)
        }
    };
    fragment.into_string()
}

pub fn html_success_fragment(message: &str) -> String {
    let fragment = html! {
        div style="padding: 1rem; margin-top: 1rem; background-color: #d4edda; color: #155724; border: 1px solid #c3e6cb; border-radius: 4px;" {
            strong { "Success! " } (message)
            p style="margin-top: 0.5rem;" {
                a href="/login" { "Click to login" }
            }
        }
    };
    fragment.into_string()
}
