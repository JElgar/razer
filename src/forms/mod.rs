enum FormFieldType {
}

struct FormField {
    readProperties: ,
    writeProperties: ,
}

struct StringFormField {
    validation: StringFieldValidation,
}

struct StringFieldValidation {
    max_length: i32,
    min_length: i32,
}

enum ReadProperties<TValidationProperties> {
    Hidden,
    Visible,
}

enum ReadProperties<TValidationProperties> {
    FunctionValue,
    UserInput(TValidationProperties)
}

struct ReadOnlyFormField {
}
