use lib::core::mem;

use bit_field::{
    BitField,
    BitArray,
};

use types::*;

use {
    UavcanStruct,
    UavcanPrimitiveType,
    DynamicArrayLength,
    DynamicArray,
};

#[derive(Debug)]
pub enum DeserializationResult {
    Finished(usize),
    BufferInsufficient(usize),
}

pub trait Deserialize {
    fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult;
}

impl Deserialize for DynamicArrayLength {
    fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
        if buffer.bit_length() + start_bit < self.bit_length {
            DeserializationResult::BufferInsufficient(0)
        } else {
            self.current_length.set_bits(start_bit as u8..self.bit_length as u8, buffer.pop_bits(self.bit_length-start_bit) as usize);
            DeserializationResult::Finished(self.bit_length)
        }
    }
}

macro_rules! impl_deserialize_for_primitive_type {
    ($type:ident) => {
        impl Deserialize for $type {
            fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                if buffer.bit_length() + start_bit < Self::bit_length() {
                    DeserializationResult::BufferInsufficient(0)
                } else {
                    self.set_bits(start_bit..Self::bit_length(), buffer.pop_bits(Self::bit_length()-start_bit));
                    DeserializationResult::Finished(Self::bit_length())
                }
            }
        }
        
        
    };
}

impl_deserialize_for_primitive_type!(Uint2);
impl_deserialize_for_primitive_type!(Uint3);
impl_deserialize_for_primitive_type!(Uint4);
impl_deserialize_for_primitive_type!(Uint5);
    
impl_deserialize_for_primitive_type!(Uint7);
impl_deserialize_for_primitive_type!(Uint8);

impl_deserialize_for_primitive_type!(Uint16);

impl_deserialize_for_primitive_type!(Uint32);

impl_deserialize_for_primitive_type!(Float16);


macro_rules! impl_deserialize_for_dynamic_array {
    ($type:ident) => {
        impl<T: UavcanPrimitiveType + Deserialize> Deserialize for $type<T> {
            fn deserialize(&mut self, start_bit: usize, buffer: &mut DeserializationBuffer) -> DeserializationResult {
                let mut bits_deserialized: usize = 0;
                
                // deserialize length
                if start_bit < self.length().bit_length {
                    match self.length().deserialize(start_bit, buffer) {
                        DeserializationResult::Finished(bits) => bits_deserialized += bits,
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),
                    }
                }
                
                let mut start_element = (start_bit + bits_deserialized - Self::length_bit_length()) / Self::element_bit_length();
                let start_element_bit = (start_bit + bits_deserialized - Self::length_bit_length()) % Self::element_bit_length();

                // first get rid of the odd bits
                if start_element_bit != 0 {
                    match self[start_element].deserialize(start_element_bit, buffer) {
                        DeserializationResult::Finished(bits) => bits_deserialized += bits,
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),
                    }
                    start_element += 1;
                }

                for i in start_element..self.length().current_length {
                    match self[i].deserialize(0, buffer) {
                        DeserializationResult::Finished(bits) => bits_deserialized += bits,
                        DeserializationResult::BufferInsufficient(bits) => return DeserializationResult::BufferInsufficient(bits_deserialized + bits),                        
                    }
                }

                DeserializationResult::Finished(bits_deserialized)
            }
            
        
        }
        
    };
}

impl_deserialize_for_dynamic_array!(DynamicArray3);
impl_deserialize_for_dynamic_array!(DynamicArray4);
impl_deserialize_for_dynamic_array!(DynamicArray5);
impl_deserialize_for_dynamic_array!(DynamicArray6);
impl_deserialize_for_dynamic_array!(DynamicArray7);
impl_deserialize_for_dynamic_array!(DynamicArray8);
impl_deserialize_for_dynamic_array!(DynamicArray9);
impl_deserialize_for_dynamic_array!(DynamicArray10);
impl_deserialize_for_dynamic_array!(DynamicArray11);
impl_deserialize_for_dynamic_array!(DynamicArray12);
impl_deserialize_for_dynamic_array!(DynamicArray13);
impl_deserialize_for_dynamic_array!(DynamicArray14);
impl_deserialize_for_dynamic_array!(DynamicArray15);
impl_deserialize_for_dynamic_array!(DynamicArray16);

impl_deserialize_for_dynamic_array!(DynamicArray31);
impl_deserialize_for_dynamic_array!(DynamicArray32);

impl_deserialize_for_dynamic_array!(DynamicArray90);



pub struct Deserializer<T: UavcanStruct> {
    structure: T,
    current_field_index: usize,
    current_type_index: usize,
    buffer: DeserializationBuffer,
}

struct DeserializationBuffer {
    buffer: [u8; 15],
    buffer_end_bit: usize,
}

impl DeserializationBuffer {
    fn new() -> Self { DeserializationBuffer{buffer: [0;15], buffer_end_bit: 0} }
        
    fn bit_length(&self) -> usize { self.buffer_end_bit }
    
    fn pop_bits(&mut self, bit_length: usize) -> u64 {
        assert!(bit_length <= 64);
        assert!(bit_length <= self.buffer_end_bit);
        
        let mut bits = 0u64;
        let mut current_bit: usize = 0;
        while current_bit < bit_length {
            if current_bit + 8 < bit_length {
                bits.set_bits(current_bit as u8..current_bit as u8 + 8, self.buffer.get_bits(current_bit..current_bit+8) as u64);
                current_bit = current_bit + 8;
            } else {
                bits.set_bits(current_bit as u8..bit_length as u8, self.buffer.get_bits(current_bit..bit_length) as u64);
                current_bit = bit_length;
            }
        }

        current_bit = 0;
        while current_bit < self.buffer_end_bit-bit_length {
            if current_bit + 8 < self.buffer_end_bit-bit_length {
                let bitmap = self.buffer.get_bits(current_bit+bit_length..current_bit+bit_length+8);
                self.buffer.set_bits(current_bit..current_bit+8, bitmap);
                current_bit = current_bit + 8;
            } else {
                let bitmap = self.buffer.get_bits(current_bit+bit_length..self.buffer_end_bit);
                self.buffer.set_bits(current_bit..self.buffer_end_bit-bit_length, bitmap);
                current_bit = self.buffer_end_bit-bit_length;
            }
        }
        
        self.buffer_end_bit -= bit_length;
        return bits;
    }
    
    fn push(&mut self, tail: &[u8]) {
        for byte in tail {
            self.buffer.set_bits(self.buffer_end_bit..self.buffer_end_bit+8, *byte);
            self.buffer_end_bit += 8;
        }
    }
    
}


impl<T: UavcanStruct> Deserializer<T> {
    pub fn new() -> Deserializer<T> {
        let structure: T;
        unsafe {
            structure = mem::zeroed();
        };            
        Deserializer{structure: structure, current_field_index: 0, current_type_index: 0, buffer: DeserializerQueue::new()}
    }

    pub fn deserialize(mut self, input: &[u8]) -> Result<Deserializer<T>, DeserializerError> {
                
        for chunk in input.chunks(8) {
            self.buffer.push(chunk);

            loop {
                
                if self.current_field_index < self.structure.flattened_fields_len() {
                    if self.current_type_index < self.structure.field(self.current_field_index).length() {
                        
                        let field_length = self.structure.field(self.current_field_index).bit_array(self.current_type_index).bit_length();
                        if field_length <= self.buffer.bit_length() {
                            self.structure.field_as_mut(self.current_field_index).bit_array_as_mut(self.current_type_index).set_bits(0..field_length, self.buffer.pop_bits(field_length));
                            self.current_type_index += 1;
                        } else {
                            break;
                        }
                    } else {
                        self.current_type_index = 0;
                        self.current_field_index += 1;
                    }
                } else {
                    if self.buffer.bit_length() >= 8 {
                        return Err(DeserializerError::StructureExhausted);
                    } else {
                        return Ok(self);
                    }
                }

            }

        }
        return Ok(self);
    }

    pub fn into_structure(self) -> Result<T, DeserializerError> {
        let number_of_fields = self.structure.flattened_fields_len();
        let finished_parsing = number_of_fields == self.current_field_index;
        if finished_parsing {
            Ok(self.structure)
        } else {
            Err(DeserializerError::NotFinished)
        }
    }
}


#[cfg(test)]
mod tests {

    use {
        UavcanStruct,
        UavcanField,
        AsUavcanField,
    };

    use deserializer::{
        Deserializer,
    };
    
    use types::{
        Uint2,
        Uint3,
        Uint8,
        Uint16,
        Uint32,
    };
    
    #[test]
    fn uavcan_parse_test_byte_aligned() {

        #[derive(UavcanStruct)]
        struct Message {
            v1: Uint8,
            v2: Uint32,
            v3: Uint16,
            v4: Uint8,
        }

        let mut deserializer: Deserializer<Message> = Deserializer::new();

        deserializer = deserializer.deserialize(&[17, 19, 0, 0, 0, 21, 0, 23]).unwrap();

        let parsed_message = deserializer.into_structure().unwrap();

        
        assert_eq!(parsed_message.v1, 17.into());
        assert_eq!(parsed_message.v2, 19.into());
        assert_eq!(parsed_message.v3, 21.into());
        assert_eq!(parsed_message.v4, 23.into());
    }




    #[test]
    fn uavcan_parse_test_misaligned() {
        
        
        #[derive(UavcanStruct)]
        struct NodeStatus {
            uptime_sec: Uint32,
            health: Uint2,
            mode: Uint3,
            sub_mode: Uint3,
            vendor_specific_status_code: Uint16,
        }

        
        let mut deserializer: Deserializer<NodeStatus> = Deserializer::new();

        deserializer = deserializer.deserialize(&[1, 0, 0, 0, 0b10001110, 5, 0]).unwrap();

        let parsed_message = deserializer.into_structure().unwrap();
        

        assert_eq!(parsed_message.uptime_sec, 1.into());
        assert_eq!(parsed_message.health, 2.into());
        assert_eq!(parsed_message.mode, 3.into());
        assert_eq!(parsed_message.sub_mode, 4.into());
        assert_eq!(parsed_message.vendor_specific_status_code, 5.into());
        
    }
}

