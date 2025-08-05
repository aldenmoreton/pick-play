pub fn m(site_key: &str) -> maud::Markup {
    super::base(
        Some("Sign Up"),
        None,
        Some(maud::html!(
            (crate::view::alertify())
            script src="https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback" defer {}
            script {
                "window.onloadTurnstileCallback = function () {
                turnstile.render('#cf-turnstile-container', {
                    sitekey: '"(site_key)"',
                    callback: function(token) {
                        document.getElementById('submit-button').disabled = false;
                    },
                    theme: 'light',
                    action: 'signup',
                });
            };"
            }
        )),
        None,
        Some(maud::html!(
                div class="flex flex-col items-center justify-center pt-10" {
                    div class="w-full max-w-xs" {
                        form hx-post="/signup" hx-on--after-on-load="if (event.detail.xhr.status !== 200) {document.getElementById('submit-button').disabled = true;turnstile.reset('#cf-turnstile-container');}" {
                        div class="px-8 pt-6 pb-8 mb-4 bg-white rounded shadow-md" {
                            div class="mb-4" {
                                label class="block mb-2 text-sm font-bold text-gray-700" for="username" {
                                    "Username"
                                }
                                input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Username";
                            }
                            div class="mb-6" {
                                label class="block mb-2 text-sm font-bold text-gray-700" for="password" { "Password" }
                                input class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline" id="password" name="password" type="password" placeholder="Password";

                                label class="block mb-2 text-sm font-bold text-gray-700" for="password_confirmation" { "Confirm Password" }
                                input id="password_confirmation" name="password_confirmation" type="password" placeholder="Password" class="w-full px-3 py-2 mb-3 leading-tight text-gray-700 border rounded shadow appearance-none focus:outline-none focus:shadow-outline";
                            }
                            button disabled id="submit-button" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit" {
                                "Sign Up"
                            }
                        }
                        div id="cf-turnstile-container" {}
                        }
                    div class="pt-3 text-sm font-bold" {
                        p { "Already have an account?" }
                        a class="text-green-500 hover:text-green-800" href="/login" { "Sign In" }
                    }
                    }
                }
        )),
        None,
    )
}
