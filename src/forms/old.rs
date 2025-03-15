use std::marker::PhantomData;

// struct FormDefinition {
// }
// 
// struct FormFieldDefinition {
// }
// 
// struct MyThing {
//     id: i32,
//     body: String,
// }
// 
// enum FormFieldWidget {
//     Hidden,
//     ReadOnly,
//     Input(FormFieldInputWidget),
// }
// 
// struct FormFieldWidget {
//     read_only: todo!(),
//     input: todo!(),
// }
//

// I like this idea but it very hard to validate that all field elemets are included in the form...
// The alternative is the go with a simple struct and then handle the positioning separately - not
// sure how we can handle positioning in a type safe way (i.e. ensureing every field is included)?
enum FormElement {
    Group(Vec<FormElement>),
    Row(Vec<FormElement>),
    Field(FormField),
    Button,
}

struct FormField {
    // TODO Static string?
    field_name: String,
    widget: FormWidget
}

enum FormWidget {
    TextInput,
}

fn build_form() {
    let a = FormElement::Group(vec![
        FormElement::Row(
            vec![
                FormElement::Field(
                    FormField {
                        widget: FormWidget::TextInput,
                        field_name: "test".into(),
                    },
                )
            ],
        ),
    ]);
}



struct MyModelFormBuilder<> {
    endpoint: &'static str,
    fields: FormField,
}

struct NoValue;
struct YesValue;

struct MyType {
    a: String,
    b: i32,
    c: Option<i32>,
}

struct MyTypeFormBuilder<Ta, Tb, Tc> {
    __marker_ta: PhantomData<Ta>,
    __marker_tb: PhantomData<Tb>,
    __marker_tc: PhantomData<Tc>,

    fields: Vec<FormField>,
}

struct MyTypeFormGroupBuilder<Ta, Tb, Tc> {
    __marker_ta: PhantomData<Ta>,
    __marker_tb: PhantomData<Tb>,
    __marker_tc: PhantomData<Tc>,

    fields: Vec<FormField>,
}

impl MyTypeFormBuilder<(), (), ()> {
    fn new() -> MyTypeFormBuilder<NoValue, NoValue, NoValue> {
        return MyTypeFormBuilder {
            __marker_ta: PhantomData,
            __marker_tb: PhantomData,
            __marker_tc: PhantomData,

            fields: vec![],
        }
    }

    // TODO Return an add method which when called udpated the form builder generics??
    // i.e. we don't have a group builder but just apply the building to the group
    // ^This wont work because if we want a group builder then we can't return then
    // top level form builder because then you lose access to updating the group
    fn add_group() {
        struct GroupData {
        }

        impl GroupData {
            fn set_a() {
            }
        }
    }
}

impl MyTypeFormBuilder<String, i32, Option<i32>> {
    fn build() {
    }
}

trait MyTypeAddGroup<Ta, Tb, Tc, Tga, Tgb, Tgc> {
    fn add_group(group: MyTypeFormGroupBuilder<Tga, Tgb, Tgc>) -> MyTypeFormBuilder<Ta, Tb, Tc>;
}

// impl MyTypeFormBuilder<NoValue, NoValue, NoValue> {
//     fn add_group<Tga, Tgb, Tgc>(group: MyTypeFormGroupBuilder<Tga, Tgb, Tgc>) -> MyTypeFormBuilder<Tga, Tgb, Tgc> {
//         todo!()
//     }
// }

impl <Tb, Tc> MyTypeFormBuilder<NoValue, Tb, Tc> {
    fn set_a(self, a: String) -> MyTypeFormBuilder<String, Tb, Tc> {
        let mut fields = self.fields;
        // fields.push(value);

        return MyTypeFormBuilder {
            __marker_ta: PhantomData,
            __marker_tb: PhantomData,
            __marker_tc: PhantomData,

            fields,
        }
    }

    // TODO This wont work - if Tb is not no value then we can't be allowing Tgb to overwrite Tb
    fn add_group<Tgb, Tgc>(group: MyTypeFormGroupBuilder<NoValue, Tgb, Tgc>) -> MyTypeFormBuilder<String, Tgb, Tgc> {
        todo!()
    }
}

fn test_thing() {
    // let a = MyTypeFormBuilder::new().set_a("abc".into()).set_a("def".into());
    let a = MyTypeFormBuilder::new().set_a("abc".into());
}

// Plan:
//
//
