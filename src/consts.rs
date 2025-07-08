use tokio_modbus::Address;

const IR_BATTERY_VOLTAGE_ADDRESS: Address = 0x11;
const IR_OUTPUT_VOLTAGE_ADDRESS: Address = 0x16;
const IR_OUTPUT_FREQUENCY_ADDRESS: Address = 0x23;

const IR_PV_VOLTAGE_ADDRESS: Address = 0x19;
const IR_TEMPERATURE_1_ADDRESS: Address = 0x19;
const IR_TEMPERATURE_2_ADDRESS: Address = 0x1a;