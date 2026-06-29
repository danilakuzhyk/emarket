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
                        li { a href="/register/customer" { "Client registration" } }
                        li { a href="/register/vendor" { "Vendor registration" } }
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
            div id="error-container" {}
        }
    };
    form.into_string()
}

pub fn customer_register_form() -> String {
    let form = html! {
        article {
            header { "New customer registration" }
            form hx-post="/api/users/customers/register" hx-target="#error-container" hx-swap="innerHTML" {
                label for="first_name" { "Name" }
                input type="text" id="first_name" name="first_name" placeholder="Ivan" required;

                label for="second_name" { "Surname" }
                input type="text" id="second_name" name="second_name" placeholder="Ivanov" required;

                label for="email" { "Email" }
                input type="email" id="email" name="email" placeholder="ivan@example.com" required;

                label for="password" { "Password" }
                input type="password" id="password" name="password" required;

                button type="submit" { "Confirm" }
            }
            div id="error-container" {}
        }
    };
    form.into_string()
}

pub fn vendor_register_form() -> String {
    let form = html! {
        article {
            header { "New vendor registration" }
            form hx-post="/api/users/vendors/register" hx-target="#error-container" hx-swap="innerHTML" {
                label for="first_name" { "Name" }
                input type="text" id="first_name" name="first_name" placeholder="Ivan" required;

                label for="second_name" { "Surname" }
                input type="text" id="second_name" name="second_name" placeholder="Ivanov" required;

                label for="email" { "Work email" }
                input type="email" id="email" name="email" placeholder="vendor@example.com" required;

                label for="password" { "Password" }
                input type="password" id="password" name="password" required;

                button type="submit" class="secondary" { "Confirm" }
            }
            div id="error-container" {}
        }
    };
    form.into_string()
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
