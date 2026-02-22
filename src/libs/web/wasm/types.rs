//! WASM type bridge — maps Rust types to WASM ABI types and JS conversions.

use crate::core::volkiwithstds::collections::{String, Vec};

/// A WASM-level ABI type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    Void,
}

/// How a Rust type maps to the WASM boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmAbi {
    /// Direct pass-through (i32, u32, bool, f32, f64, i64, u64).
    Direct(WasmType),
    /// String types (&str, String) flatten to (ptr: i32, len: i32).
    StringPair,
    /// Unit type () maps to void.
    Void,
}

/// A single parameter in the WASM signature.
#[derive(Debug, Clone)]
pub struct WasmParam {
    pub name: String,
    pub rust_type: String,
    pub abi: WasmAbi,
}

/// Full WASM function signature (flattened to ABI types).
#[derive(Debug, Clone)]
pub struct WasmSignature {
    pub name: String,
    pub params: Vec<WasmParam>,
    pub ret: WasmAbi,
}

/// Map a Rust type string to its WASM ABI representation.
pub fn rust_type_to_wasm(ty: &str) -> WasmAbi {
    match ty {
        "i32" | "u32" | "bool" => WasmAbi::Direct(WasmType::I32),
        "i64" | "u64" => WasmAbi::Direct(WasmType::I64),
        "f32" => WasmAbi::Direct(WasmType::F32),
        "f64" => WasmAbi::Direct(WasmType::F64),
        "&str" | "String" => WasmAbi::StringPair,
        "()" => WasmAbi::Void,
        _ => WasmAbi::Direct(WasmType::I32), // default: treat as i32 handle
    }
}

/// Get the WASM type name string for code generation.
pub fn wasm_type_str(wt: WasmType) -> &'static str {
    match wt {
        WasmType::I32 => "i32",
        WasmType::I64 => "i64",
        WasmType::F32 => "f32",
        WasmType::F64 => "f64",
        WasmType::Void => "()",
    }
}

/// Return the JS type conversion expression for reading a WASM value.
pub fn js_from_wasm(abi: &WasmAbi) -> &'static str {
    match abi {
        WasmAbi::Direct(WasmType::I32) => "/* direct i32 */",
        WasmAbi::Direct(WasmType::I64) => "BigInt",
        WasmAbi::Direct(WasmType::F32) | WasmAbi::Direct(WasmType::F64) => "/* direct float */",
        WasmAbi::Direct(WasmType::Void) | WasmAbi::Void => "undefined",
        WasmAbi::StringPair => "/* string via linear memory */",
    }
}

/// Build a `WasmSignature` from scanner output.
pub fn build_signature(
    name: &str,
    params: &[(String, String)],
) -> WasmSignature {
    let mut wasm_params = Vec::new();
    for (pname, pty) in params {
        wasm_params.push(WasmParam {
            name: pname.clone(),
            rust_type: pty.clone(),
            abi: rust_type_to_wasm(pty.as_str()),
        });
    }
    WasmSignature {
        name: String::from(name),
        params: wasm_params,
        ret: WasmAbi::Void, // Client functions always return void
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_type_to_wasm_i32() {
        assert_eq!(rust_type_to_wasm("i32"), WasmAbi::Direct(WasmType::I32));
        assert_eq!(rust_type_to_wasm("u32"), WasmAbi::Direct(WasmType::I32));
        assert_eq!(rust_type_to_wasm("bool"), WasmAbi::Direct(WasmType::I32));
    }

    #[test]
    fn test_rust_type_to_wasm_i64() {
        assert_eq!(rust_type_to_wasm("i64"), WasmAbi::Direct(WasmType::I64));
        assert_eq!(rust_type_to_wasm("u64"), WasmAbi::Direct(WasmType::I64));
    }

    #[test]
    fn test_rust_type_to_wasm_float() {
        assert_eq!(rust_type_to_wasm("f32"), WasmAbi::Direct(WasmType::F32));
        assert_eq!(rust_type_to_wasm("f64"), WasmAbi::Direct(WasmType::F64));
    }

    #[test]
    fn test_rust_type_to_wasm_string() {
        assert_eq!(rust_type_to_wasm("&str"), WasmAbi::StringPair);
        assert_eq!(rust_type_to_wasm("String"), WasmAbi::StringPair);
    }

    #[test]
    fn test_rust_type_to_wasm_void() {
        assert_eq!(rust_type_to_wasm("()"), WasmAbi::Void);
    }

    #[test]
    fn test_build_signature() {
        let params = [
            (String::from("target"), String::from("&str")),
            (String::from("count"), String::from("i32")),
        ];
        let sig = build_signature("on_click", &params);
        assert_eq!(sig.name.as_str(), "on_click");
        assert_eq!(sig.params.len(), 2);
        assert_eq!(sig.params[0].abi, WasmAbi::StringPair);
        assert_eq!(sig.params[1].abi, WasmAbi::Direct(WasmType::I32));
        assert_eq!(sig.ret, WasmAbi::Void);
    }

    #[test]
    fn test_wasm_type_str() {
        assert_eq!(wasm_type_str(WasmType::I32), "i32");
        assert_eq!(wasm_type_str(WasmType::I64), "i64");
        assert_eq!(wasm_type_str(WasmType::F32), "f32");
        assert_eq!(wasm_type_str(WasmType::F64), "f64");
        assert_eq!(wasm_type_str(WasmType::Void), "()");
    }
}
