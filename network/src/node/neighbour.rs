use serde::de::{self, Deserialize, Deserializer, MapAccess, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use std::fmt;
use thiserror::Error;
use uuid::Uuid;

/// Represents the role a node can have in the network
#[derive(Clone, PartialEq, Copy)]
pub enum Role {
    /// A tracker node that helps other nodes discover the network
    Tracker,
    /// A regular peer node in the network
    Node,
    /// A miner node that can create new blocks
    Miner,
}

/// Error returned when an unknown protocol value is received
#[derive(Error, Debug, derive_more::From)]
pub enum WrongProtocolError {
    /// The protocol value is not recognized
    UnknownProtocol {
        /// Unknown protocol number.
        protocol: u32,
    },
}

impl fmt::Display for WrongProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unknow Role protocol.")
    }
}

impl Role {
    /// Converts a Role to its protocol representation
    ///
    /// # Returns
    /// The protocol number for this role
    #[must_use]
    pub fn to_protocol(&self) -> u32 {
        match self {
            Role::Tracker => 0,
            Role::Node => 1,
            Role::Miner => 2,
        }
    }

    /// Creates a Role from its protocol representation
    ///
    /// # Arguments
    /// * `protocol` - The protocol number
    ///
    /// # Returns
    /// The corresponding Role or an error if the protocol is unknown
    pub fn from_protocol(protocol: u32) -> Result<Self, WrongProtocolError> {
        match protocol {
            0 => Ok(Role::Tracker),
            1 => Ok(Role::Node),
            2 => Ok(Role::Miner),
            _ => Err(WrongProtocolError::UnknownProtocol { protocol }),
        }
    }
}

/// Represents a neighbor node in the network
#[derive(Clone)]
pub struct Neighbour {
    /// Unique identifier for this neighbor
    pub id: Uuid,
    /// Network address of this neighbor
    pub address: String,
    /// Role of this neighbor in the network
    pub role: Role,
}

impl PartialEq for Neighbour {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl fmt::Debug for Neighbour {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Neighbour")
            .field("id", &self.id.to_string())
            .field("address", &self.address)
            .field("role", &self.role.to_protocol())
            .finish()
    }
}

impl Serialize for Neighbour {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Neighbour", 3)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("address", &self.address)?;
        s.serialize_field("role", &self.role.to_protocol())?;
        s.end()
    }
}

struct RoleVisitor;

#[allow(clippy::elidable_lifetime_names)]
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
            Err(e) => Err(E::custom(format!("{e}"))),
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
        enum Field {
            Id,
            Address,
            Role,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                #[allow(clippy::elidable_lifetime_names)]
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
                        }
                        Field::Address => {
                            if address.is_some() {
                                return Err(de::Error::duplicate_field("address"));
                            }
                            address = Some(map.next_value()?);
                        }
                        Field::Role => {
                            if role.is_some() {
                                return Err(de::Error::duplicate_field("role"));
                            }
                            let raw = map.next_value()?;
                            role = Some(Role::from_protocol(raw).map_err(|_| {
                                de::Error::unknown_variant(
                                    raw.to_string().as_str(),
                                    &["0 (Tracker)", "1 (Node)", "2 (Miner)"],
                                )
                            })?);
                        }
                    }
                }
                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let address = address.ok_or_else(|| de::Error::missing_field("address"))?;
                let role = role.ok_or_else(|| de::Error::missing_field("role"))?;
                let n = Neighbour { id, address, role };
                Ok(n)
            }
        }

        const FIELDS: &[&str] = &["id", "address", "role"];
        d.deserialize_struct("Neighbour", FIELDS, NeighbourVisitor)
    }
}
