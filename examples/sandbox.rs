use rs_vm::{
  script::Script,
  vm::{Vm, BANNER, VERSION},
};

fn main() {
  println!("{} v{}", BANNER, VERSION);
  let mut vm = Vm::default();

  vm.add_script(Script::new(
    "virtual://test_script",
    Some("test_script"),
    Some(
      "
      function hello_world() {
        println(\"Hello!\");
      }

      function hello_world2(a) {
        println(a);
      }
      
      hello_world();
      hello_world2(\"test\");",
    ),
  ));

  for it in vm.scripts().iter() {
    println!(" - {}", it)
  }

  vm.run().unwrap();
}
