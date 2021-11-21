use rs_vm::{script::Script, vm::{BANNER, VERSION, Vm}};

fn main() {
  println!("{} v{}", BANNER, VERSION);
  let mut vm = Vm::default();

  vm.scripts_mut().push(Script::new("virtual://test_script", Some("test_script"), None));

  for it in vm.scripts().iter() {
    println!(" - {}", it)
  }

  vm.run().unwrap();
}
