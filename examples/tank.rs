//! What a player's device can tell its tank in the battle arena to do.
//!
//! An [`Action`] is one thing the tank does, with whatever it takes; a
//! [`Command`] is the payload of the router cell's `tank` command, an action
//! with the name of the tank to drive. The router (`battle-arena/router`) reads
//! [`Action`] as is, and forwards it on as the display's `TankMessage`
//! (`battle-arena/web/src/messages.ts`).
//!
//! An optional field left as `None` is left out, and the arena uses its
//! default.
//!
//! Needs serde, which `m new -f` leaves out:
//!
//! ```toml
//! serde = { version = "1", default-features = false, features = ["derive"] }
//! ```
//!
//! ```ignore
//! let shot = Command { action: Action::Bounce { bounces: Some(3) }, name: PLAYER_NAME };
//! tx.send(router, "tank", &shot).await?;
//! ```

use serde::{Deserialize, Serialize};

/// Everything a tank can be told to do.
///
/// A tank starts in auto mode: it stays put and its gun sweeps round on its
/// own. Driving, turning or aiming switches it to manual, where the gun holds
/// along the hull (unless aimed) and the tank only moves when told to;
/// [`Action::Auto`] switches back.
///
/// Firing faster than about 3 shots a second for a while overheats the tank:
/// it can't do anything for 3 seconds, and every hit on it meanwhile scores ×4.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum Action {
    /// Puts the tank on a display. Any action does that for a tank not on one,
    /// so this is only for showing up without doing anything.
    Join,
    /// Fires a plain shell.
    Fire,
    /// Drives forward.
    Forward {
        /// Steps, 0.1–10; one step is about 1⅓ tank lengths. Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<f32>,
    },
    /// Drives backward.
    Backward {
        /// Steps, 0.1–10; one step is about 1⅓ tank lengths. Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        amount: Option<f32>,
    },
    /// Turns the hull on the spot.
    #[serde(rename = "turn")]
    TurnTank {
        /// From [`TurnFrom::Hull`]: how far to turn, −360–360, clockwise when
        /// positive. From [`TurnFrom::Table`]: the compass bearing to face,
        /// turning the short way round.
        degrees: f32,
        /// What `degrees` is measured from. Defaults to [`TurnFrom::Hull`].
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<TurnFrom>,
    },
    /// Swings the gun round, and holds it wherever `from` says.
    #[serde(rename = "aim")]
    AimGun {
        /// From [`AimFrom::Gun`]: how far to swing, −360–360, clockwise when
        /// positive. From [`AimFrom::Hull`]: the angle clockwise of the hull's
        /// nose. From [`AimFrom::Table`]: the compass bearing to point at.
        degrees: f32,
        /// What `degrees` is measured from. Defaults to [`AimFrom::Gun`].
        #[serde(skip_serializing_if = "Option::is_none")]
        from: Option<AimFrom>,
    },
    /// Goes back to auto mode, and forgets any aim.
    Auto,
    /// Raises a shield: shells bounce off it, and the tank can't fire while
    /// it's up.
    Shield {
        /// How long it lasts, 1–10. Defaults to 5.
        #[serde(skip_serializing_if = "Option::is_none")]
        seconds: Option<f32>,
    },
    /// Charges up, then fires a shell that bounces off the table's edge.
    Bounce {
        /// Times it bounces, 0–5; each takes 0.6 s to charge, and 0 is a plain
        /// shot. Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        bounces: Option<u8>,
    },
    /// Charges up, then fires a shell that's bigger the longer it charged.
    Charge {
        /// Seconds to charge, 0–3; 0 is a plain shot. Defaults to 1.
        #[serde(skip_serializing_if = "Option::is_none")]
        seconds: Option<f32>,
    },
    /// Takes the tank off its display, for everyone driving it.
    Leave,
}

/// What a [`Action::TurnTank`]'s angle is measured from. On the table, 0° is
/// north (the top) and angles go clockwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TurnFrom {
    /// The way the hull faces now: turn by the angle.
    Hull,
    /// The table: face the compass bearing.
    Table,
}

/// What an [`Action::AimGun`]'s angle is measured from, which also decides
/// what the gun does as the hull moves afterwards. On the table, 0° is north
/// (the top) and angles go clockwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AimFrom {
    /// Where the gun points now: swing by the angle, and keep whichever hold
    /// the gun had.
    Gun,
    /// The hull's nose: the gun turns with the hull. Aiming 0 from here puts
    /// the gun back along the hull, where every tank starts.
    Hull,
    /// The table: the gun keeps the compass bearing while the hull turns
    /// underneath it.
    Table,
}

/// The router's `tank` command: an action, and the tank to drive.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Command<'a> {
    #[serde(flatten)]
    pub action: Action,
    /// The tank to drive, and its label on the display. Every cell that sends
    /// the same name drives the same tank, so a team can share one; sending
    /// another name drives another tank.
    pub name: &'a str,
}
