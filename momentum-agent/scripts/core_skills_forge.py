import os
import yaml

TARGET_DIR = "skills/core"

def generate_authentication_skill():
    return {
        "platform": "generic_authentication",
        "intents": [
            {
                "name": "login_account",
                "trigger_roles": ["Login", "Sign In", "Authenticate", "Enter Password"],
                "strategy": [
                    "Locate login identifiers (email, username) and password fields semantically.",
                    "If a third-party SSO (Google, Apple, GitHub) is requested or required, click the SSO button.",
                    "Handle 2FA natively by prompting the user for the code."
                ],
                "actions": [
                    "Type: email, {user_email}",
                    "Type: password, {user_password}",
                    "Click: Login"
                ],
                "recovery": [
                    "If login fails due to captcha, trigger the solve_captcha intent.",
                    "If wrong password, notify user."
                ]
            },
            {
                "name": "signup_account",
                "trigger_roles": ["Sign Up", "Register", "Create Account", "Get Started"],
                "strategy": [
                    "Fill out full name, email, and password semantically.",
                    "Accept terms and conditions if a checkbox is detected."
                ],
                "actions": [
                    "Type: First Name, {first_name}",
                    "Type: Last Name, {last_name}",
                    "Type: Email, {user_email}",
                    "Type: Password, {user_password}",
                    "Click: Terms",
                    "Click: Sign Up"
                ]
            }
        ]
    }

def generate_captcha_skill():
    return {
        "platform": "generic_captcha",
        "intents": [
            {
                "name": "solve_captcha",
                "trigger_roles": ["Captcha", "Verify you are human", "I am not a robot", "Security check"],
                "strategy": [
                    "Identify the type of captcha (reCAPTCHA, hCaptcha, Cloudflare Turnstile).",
                    "For checkbox captchas, compute the center coordinate and click natively.",
                    "For visual challenges, pause execution and wait for human intervention or external API solver."
                ],
                "actions": [
                    "Scroll: down",
                    "Click: I am human",
                    "Extract: captcha_result"
                ]
            }
        ]
    }

def generate_information_gathering():
    return {
        "platform": "generic_information",
        "intents": [
            {
                "name": "background_read",
                "trigger_roles": ["Read", "Learn", "Digest", "Summarize"],
                "strategy": [
                    "Used during the Slumberless OS idle state.",
                    "Scroll through the document slowly to trigger lazy-loaded text.",
                    "Extract chunks of text and compress into embeddings for SQLite ROM."
                ],
                "actions": [
                    "Scroll: down",
                    "Scroll: down",
                    "Scroll: down",
                    "Extract: full_page_content"
                ]
            }
        ]
    }

def build_yaml(filename, data):
    path = os.path.join(TARGET_DIR, filename)
    with open(path, 'w') as f:
        yaml.dump(data, f, default_flow_style=False, sort_keys=False)
    print(f"Generated core skill blueprint: {path}")

if __name__ == "__main__":
    os.makedirs(TARGET_DIR, exist_ok=True)
    
    build_yaml("authentication.yaml", generate_authentication_skill())
    build_yaml("captcha.yaml", generate_captcha_skill())
    build_yaml("slumberless.yaml", generate_information_gathering())
    
    print("\nCore Skills Forge completed. Run the agent to see these instantly loaded into RAM.")
