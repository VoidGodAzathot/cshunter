use windows::core::PCWSTR;

pub fn string_to_pcwstr(str: String) -> PCWSTR {
    PCWSTR(str.encode_utf16().chain(Some(0)).collect::<Vec<u16>>().as_mut_ptr())
}