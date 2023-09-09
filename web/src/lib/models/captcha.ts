import type { Validator } from "./config"

export interface Captcha {
    id: string
    validator: Validator
    challenge: string
}
