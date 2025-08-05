use crate::{controllers::session::google::GoogleOauth, AppStateRef};

pub fn m(profile: GoogleOauth, state: AppStateRef) -> maud::Markup {
    super::base(
        Some("Finish Signing Up"),
        None,
        Some(maud::html! {
            script src="https://challenges.cloudflare.com/turnstile/v0/api.js?onload=onloadTurnstileCallback" defer {}
            script {
                "window.onloadTurnstileCallback = function () {
                    turnstile.render('#cf-turnstile-container', {
                        sitekey: '"(state.turnstile.site_key)"',
                        callback: function(token) {
                            document.getElementById('submit-button').disabled = false;
                        },
                        theme: 'light',
                        action: 'signup',
                    });
                };"
            }
            (crate::view::alertify())
        }),
        None,
        Some(maud::html!(
            div class="flex flex-col items-center justify-center pt-10" {
                div class="w-full max-w-xs" {
                    form
                        hx-post="/finish-signup"
                        hx-swap="afterend"
                        hx-on--after-on-load="if (event.detail.xhr.status !== 200) {document.getElementById('submit-button').disabled = true;turnstile.reset('#cf-turnstile-container');}"
                        {
                        div class="flex flex-col items-center justify-center px-8 pt-6 pb-6 mb-4 bg-white rounded shadow-md" {
                            @if let (Some(picture), Some(first), Some(last)) = (profile.extra.get("picture"), profile.extra.get("given_name"), profile.extra.get("famly_name")) {
                                img src=(picture) class="m-2";

                                div class="mb-4" {
                                    label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                        "First Name"
                                    }
                                    input disabled class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" value=(first);
                                }

                                div class="mb-4" {
                                    label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                        "Last Name"
                                    }
                                    input disabled class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" value=(last);
                                }
                            }


                            div class="mb-4" {
                                label class="block mb-2 text-sm font-bold text-left text-gray-700" for="username" {
                                    "Username"
                                }
                                input class="w-full px-3 py-2 leading-tight text-gray-700 border rounded shadow appearance-none disabled:bg-gray-200 disabled:cursor-not-allowed focus:outline-none focus:shadow-outline" id="username" name="username" type="text" placeholder="Choose Username";
                            }

                            div id="cf-turnstile-container" {}

                            button disabled id="submit-button" class="px-4 py-2 font-bold text-white bg-green-500 rounded disabled:cursor-wait disabled:bg-gray-400 hover:bg-green-700 focus:outline-none focus:shadow-outline" type="submit" style="font-size: 150%;" {
                                "Sign Up"
                            }
                        }
                    }
                }
            }
        )),
        None,
    )
}
