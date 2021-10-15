// Copyright 2018 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

pub fn parse_template<'a>(
    context: &mut ::Context,
    name: ::Cow<'a, str>,
    parameters: &[::Parameter<'a>],
) -> Option<::Template<'a>> {
    let mut named_parameters = ::HashMap::new();
    let mut unnamed_parameters = vec![];
    for parameter in parameters {
        macro_rules! warn {
            ($message:tt) => {{
                ::add_warning(context, parameter, ::WarningMessage::$message);
                return None;
            }};
        }
        match parameter.name {
            None => match ::parse_text(&parameter.value) {
                None => warn!(ValueUnrecognized),
                Some(value) => unnamed_parameters.push(value),
            },
            Some(_) => match ::parse_parameter_name(parameter) {
                None => warn!(Unrecognized),
                Some(name) => {
                    if named_parameters.contains_key(name) {
                        ::add_warning(context, parameter, ::WarningMessage::Duplicate);
                    }
                    match ::parse_text(&parameter.value) {
                        None => warn!(ValueUnrecognized),
                        Some(value) => {
                            named_parameters.insert(::Cow::Borrowed(name), value);
                        }
                    }
                }
            },
        }
    }
    Some(::Template {
        name,
        named_parameters,
        unnamed_parameters,
    })
}
