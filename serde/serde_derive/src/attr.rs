use syn::{Field, Result, Variant};

pub fn name_of_field(field: &Field) -> Result<String> {
    Ok(field.ident.as_ref().unwrap().to_string())
}

pub fn name_of_variant(var: &Variant) -> Result<String> {
    Ok(var.ident.to_string())
}
