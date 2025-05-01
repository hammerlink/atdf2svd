use crate::atdf;
use crate::chip;
use crate::util;
use crate::ElementExt;

fn update_register_group(register_group: &mut chip::RegisterGroup, delta: usize) {
    for register in register_group.registers.values_mut() {
        register.offset -= delta;
    }
    for subgroup in register_group.subgroups.iter_mut() {
        update_register_group(subgroup, delta);
    }
}

fn base_line_address(peripheral: &mut chip::Peripheral) {
    let base_address = peripheral
        .register_group
        .get_all_registers()
        .iter()
        .map(|r| r.address)
        .min();
    if base_address.is_none() {
        return;
    }
    let base_address = base_address.unwrap();
    if base_address > peripheral.address {
        let delta = base_address - peripheral.address;
        peripheral.address = base_address;
        update_register_group(&mut peripheral.register_group, delta);
    }
}

pub fn parse_list(
    el: &xmltree::Element,
    modules: &xmltree::Element,
) -> crate::Result<Vec<chip::Peripheral>> {
    let mut peripherals = vec![];

    for module in el.iter_children_with_name("module", None) {
        let module_name = module.attr("name")?;

        for instance in module.iter_children_with_name("instance", Some("module")) {
            // Find corresponding module
            let module = modules.first_child_by_attr(Some("module"), "name", module_name)?;

            // The register definitions can reference value-groups, that are stored on the same
            // level as the register-groups, so we parse them in here first.
            let value_groups = atdf::values::parse_value_groups(module)?;

            // An instance should always have one register-group
            let instance_register_group = match instance.first_child("register-group") {
                Ok(rg) => rg,
                Err(_) => continue,
            };
            let name_in_module = instance_register_group.attr("name-in-module")?;
            let address = util::parse_int(instance_register_group.attr("offset")?)?;

            let mut register_groups = atdf::register_group::parse_list(module, 0, &value_groups)?;
            let mut main_register_group = register_groups.get(name_in_module).cloned().unwrap();
            atdf::register_group::build_register_group_hierarchy(
                &mut main_register_group,
                &mut register_groups,
                address,
                0,
            )?;

            let mut peripheral = chip::Peripheral {
                name: instance.attr("name")?.clone(),
                name_in_module: name_in_module.clone(),
                description: instance
                    .attr("caption")
                    .or(module.attr("caption"))
                    .ok()
                    .cloned()
                    .and_then(|d| if !d.is_empty() { Some(d) } else { None }),
                address,
                register_group: main_register_group,
            };
            base_line_address(&mut peripheral);
            peripherals.push(peripheral)
        }
    }

    Ok(peripherals)
}
