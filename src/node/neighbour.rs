pub mod neighbour {
    
    use uuid::Uuid;
    use serde::de::{self, Deserialize, Deserializer, Visitor, MapAccess};
    use serde::ser::{Serialize, SerializeStruct, Serializer};
    use thiserror::Error;
    use std::fmt;

    #[derive(Clone, PartialEq)]
    pub enum Role {
        Tracker,
        Node,
        Miner,
    }

    #[derive(Error, Debug, derive_more::From)]
    pub enum WrongProtocolError {
        UnknownProtocol{protocol: u32},
    }

    impl fmt::Display for WrongProtocolError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Unknow Role protocol.")
        }
    }

    impl Role {
        pub fn to_protocol(&self) -> u32 {
            match self {
                Role::Tracker => 0,
                Role::Node => 1,
                Role::Miner => 2,
            }
        }

        pub fn from_protocol(protocol: u32) -> Result<Self, WrongProtocolError> {
            match protocol {
                0 => Ok(Role::Tracker),
                1 => Ok(Role::Node),
                2 => Ok(Role::Miner),
                _ => Err(WrongProtocolError::UnknownProtocol{protocol: protocol}),
            }
        }
    }

    #[derive(Clone)]
    pub struct Neighbour {
        pub id: Uuid,
        pub address: String,
        pub role: Role,
    }

    impl PartialEq for Neighbour {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id
        }
    }

    impl Serialize for Neighbour {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where S: Serializer,
        {
            let mut s = serializer.serialize_struct("Neighbour", 3)?;
            s.serialize_field("id", &self.id.to_string())?;
            s.serialize_field("address", &self.address)?;
            s.serialize_field("role", &self.role.to_protocol())?;
            s.end()
        }
    }

    struct RoleVisitor;
    
    impl<'de> Visitor<'de> for RoleVisitor {
        type Value = Role;
    
        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("Neighbour Role")
        }
    
        fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            match Role::from_protocol(value) {
                Ok(v) => Ok(v),
                Err(e) => Err(E::custom(format!("e"))), 
            }
        }
    }

    impl<'de> Deserialize<'de> for Role {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_u32(RoleVisitor)
        }
    }


    impl<'de> Deserialize<'de> for Neighbour {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {

            enum Field { Id, Address, Role }

            impl<'de> Deserialize<'de> for Field {
                fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
                where
                    D: Deserializer<'de>,
                {
                    struct FieldVisitor;

                    impl<'de> Visitor<'de> for FieldVisitor {
                        type Value = Field;

                        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                            formatter.write_str("`id`, `address` or `role`")
                        }

                        fn visit_str<E>(self, value: &str) -> Result<Field, E>
                        where
                            E: de::Error,
                        {
                            match value {
                                "id" => Ok(Field::Id),
                                "address" => Ok(Field::Address),
                                "role" => Ok(Field::Role),
                                _ => Err(de::Error::unknown_field(value, FIELDS)),
                            }
                        }
                    }

                    deserializer.deserialize_identifier(FieldVisitor)
                }
            }

            struct NeighbourVisitor;


            impl<'de> Visitor<'de> for NeighbourVisitor {
                type Value = Neighbour;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    formatter.write_str("An owned String")
                }

                fn visit_map<V>(self, mut map: V) -> Result<Neighbour, V::Error> 
                where
                    V: MapAccess<'de>,
                {
                    let mut id = None;
                    let mut address = None;
                    let mut role = None;

                    while let Some(key) = map.next_key()? {
                        match key {
                            Field::Id => {
                                if id.is_some() {
                                     return Err(de::Error::duplicate_field("id"));
                                }
                                id = Some(map.next_value()?);
                            },
                            Field::Address => {
                                if address.is_some() {
                                     return Err(de::Error::duplicate_field("address"));
                                }
                                address = Some(map.next_value()?);
                            },
                            Field::Role => {
                                if role.is_some() {
                                     return Err(de::Error::duplicate_field("role"));
                                }
                                role = Some(map.next_value()?);
                            },
                        }
                    }
                    let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                    let address = address.ok_or_else(|| de::Error::missing_field("address"))?;
                    let role = role.ok_or_else(|| de::Error::missing_field("role"))?;
                    Ok(Neighbour {
                        id,
                        address,
                        role,
                    })
                }
            }
                            
            const FIELDS: &[&str] = &["id", "address", "role"];
            d.deserialize_struct("Neighbour", FIELDS, NeighbourVisitor)
        }
    }
}
