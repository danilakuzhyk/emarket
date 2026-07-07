use maud::html;

pub fn html_error_fragment(message: &str) -> String {
    let fragment = html! {
        div style="padding: 1rem; margin-top: 1rem; background-color: #f8d7da; color: #721c24; border: 1px solid #f5c6cb; border-radius: 4px;" {
            strong { "Error: " } (message)
        }
    };
    fragment.into_string()
}