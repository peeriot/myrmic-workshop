use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    
    
    Join,
    
    Fire,
    
    Forward {
        
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<f32>,
    },
    
    Backward {
        
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<f32>,
    },
    
    #[serde(rename = "turn")]
    TurnTank {
        
        
        
        degrees: f32,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<TurnFrom>,
    },
    
    #[serde(rename = "aim")]
    AimGun {
        
        
        
        degrees: f32,
        
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<AimFrom>,
    },
    
    Auto,
    
    Shield {
        
        #[serde(skip_serializing_if = "Option::is_none")]
        seconds: Option<f32>,
    },
    
    Bounce {
        
        
        #[serde(skip_serializing_if = "Option::is_none")]
        bounces: Option<u8>,
    },
    
    Charge {
        
        #[serde(skip_serializing_if = "Option::is_none")]
        seconds: Option<f32>,
    },
    
    Leave,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnFrom {
    
    Hull,
    
    Table,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AimFrom {
    
    
    Gun,
    
    
    Hull,
    
    
    Table,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Command<'a> {
    #[serde(flatten)]
    pub action: Action,
    
    pub player_name: &'a str,
}
