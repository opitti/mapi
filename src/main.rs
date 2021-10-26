use derive_macro::{Describe, my_custom_attribute,trace_var};

#[my_custom_attribute]
struct S{}


#[derive(Describe)]
struct MyStruct {
    my_string: String,
    my_enum: MyEnum,
    my_number: f64,
}

#[derive(Describe)]
struct MyTupleStruct(u32, String, i8);

#[derive(Describe)]
enum MyEnum {
    VariantA,
    VariantB,
}

#[derive(Describe)]
union MyUnion {
    unsigned: u32,
    signed: i32,
}

#[trace_var(a)]
pub fn something(){
  let mut a=9;
  a=6;
  a=0;
}

fn try_2(){
    println!("try");
}

fn main() {
    MyStruct::describe();
    MyTupleStruct::describe();
    MyEnum::describe();
    MyUnion::describe();

    let demo=H{};
    try_2();
    something();
}