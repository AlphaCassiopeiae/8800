import skip
import skip.pcbnew
import skip.pcbnew.net

def rename_noconnect_nets(schematic_path):
    schem = skip.Schematic('samp/my_schem.kicad_sch')

    schem.symbol.
    
    # Dictionary to track nets with only No-Connects
    no_connect_nets = {}
    
    for net in doc.read():
        no_connect_pins = [pin for pin in net.pins if pin.is_no_connect]
        
        if len(no_connect_pins) == len(net.pins):  # All pins are No-Connect
            new_name = f"dimm-{no_connect_pins[0].name}"
            net.name = new_name
            
            # Remove No-Connect flags
            for pin in no_connect_pins:
                pin.is_no_connect = False
    
    doc.save(schematic_path)
    print(f"Updated schematic saved: {schematic_path}")

# Example usage
rename_noconnect_nets("backplane.kicad_sch")
