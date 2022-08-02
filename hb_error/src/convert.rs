/// The ConvertFrom trait is based on the From trait and created to allow the
/// implementation of conversion between Results where the Error type implements
/// the From trait to convert to the new Error type.
pub trait ConvertFrom<T> {
    fn convert_from(_: T) -> Self;
}

/// Default implementation of the ConvertFrom trait for any Result that the
/// Error types implementing the From<> trait.
impl<Val, EFrom, ETo: From<EFrom>> ConvertFrom<Result<Val, EFrom>> for Result<Val, ETo> {
    fn convert_from(f: Result<Val, EFrom>) -> Self {
        match f {
            Err(error) => Err(error.into()),
            Ok(o) => Ok(o),
        }
    }
}

/// The ConvertInto trait is based on the Into trait which is used to allow
/// the conversion between Results that implement [ConvertFrom].
pub trait ConvertInto<T> {
    fn convert(self) -> T;
}

/// Default implementation of the ConvertTo trait for any Result that
/// that implement the [ConvertFrom] trait.
impl<Val, EFrom, ETo> ConvertInto<Result<Val, ETo>> for Result<Val, EFrom>
where
    Result<Val, ETo>: ConvertFrom<Result<Val, EFrom>>,
{
    fn convert(self) -> Result<Val, ETo> {
        Result::<Val, ETo>::convert_from(self)
    }
}
