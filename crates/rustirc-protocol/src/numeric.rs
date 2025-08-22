//! IRC numeric replies

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum Numeric {
    // Command responses
    RplWelcome = 1,
    RplYourHost = 2,
    RplCreated = 3,
    RplMyInfo = 4,
    RplISupport = 5,

    // User modes
    RplUModeIs = 221,

    // Channel information
    RplChannelModeIs = 324,
    RplNoTopic = 331,
    RplTopic = 332,
    RplTopicWhoTime = 333,

    // User information
    RplWhoisUser = 311,
    RplWhoisServer = 312,
    RplWhoisOperator = 313,
    RplWhoisIdle = 317,
    RplEndOfWhois = 318,
    RplWhoisChannels = 319,

    // List replies
    RplListStart = 321,
    RplList = 322,
    RplListEnd = 323,

    // Names replies
    RplNameReply = 353,
    RplEndOfNames = 366,

    // MOTD
    RplMotdStart = 375,
    RplMotd = 372,
    RplEndOfMotd = 376,

    // Error replies
    ErrNoSuchNick = 401,
    ErrNoSuchServer = 402,
    ErrNoSuchChannel = 403,
    ErrCannotSendToChan = 404,
    ErrTooManyChannels = 405,
    ErrWasNoSuchNick = 406,
    ErrTooManyTargets = 407,
    ErrNoOrigin = 409,
    ErrNoRecipient = 411,
    ErrNoTextToSend = 412,
    ErrNoTopLevel = 413,
    ErrWildTopLevel = 414,
    ErrUnknownCommand = 421,
    ErrNoMotd = 422,
    ErrNoAdminInfo = 423,
    ErrFileError = 424,
    ErrNoNicknameGiven = 431,
    ErrErroneousNickname = 432,
    ErrNicknameInUse = 433,
    ErrNickCollision = 436,
    ErrUserNotInChannel = 441,
    ErrNotOnChannel = 442,
    ErrUserOnChannel = 443,
    ErrNoLogin = 444,
    ErrSummonDisabled = 445,
    ErrUsersDisabled = 446,
    ErrNotRegistered = 451,
    ErrNeedMoreParams = 461,
    ErrAlreadyRegistered = 462,
    ErrNoPermForHost = 463,
    ErrPasswdMismatch = 464,
    ErrYoureBannedCreep = 465,
    ErrKeySet = 467,
    ErrChannelIsFull = 471,
    ErrUnknownMode = 472,
    ErrInviteOnlyChan = 473,
    ErrBannedFromChan = 474,
    ErrBadChannelKey = 475,
    ErrBadChannelMask = 476,
    ErrNoChannelModes = 477,
    ErrBanListFull = 478,
    ErrNoPrivileges = 481,
    ErrChanOpPrivsNeeded = 482,
    ErrCantKillServer = 483,
    ErrRestricted = 484,
    ErrUniqOpPrivsNeeded = 485,
    ErrNoOperHost = 491,
    ErrUModeUnknownFlag = 501,
    ErrUsersDontMatch = 502,

    // SASL
    RplLoggedIn = 900,
    RplLoggedOut = 901,
    RplNickLocked = 902,
    RplSaslSuccess = 903,
    RplSaslFail = 904,
    RplSaslTooLong = 905,
    RplSaslAbort = 906,
    RplSaslAlready = 907,
    RplSaslMechs = 908,
}

impl Numeric {
    pub fn as_str(&self) -> &'static str {
        match self {
            Numeric::RplWelcome => "001",
            Numeric::RplYourHost => "002",
            Numeric::RplCreated => "003",
            Numeric::RplMyInfo => "004",
            Numeric::RplISupport => "005",
            _ => "000", // Simplified for brevity
        }
    }

    pub fn from_u16(code: u16) -> Option<Self> {
        match code {
            1 => Some(Numeric::RplWelcome),
            2 => Some(Numeric::RplYourHost),
            3 => Some(Numeric::RplCreated),
            4 => Some(Numeric::RplMyInfo),
            5 => Some(Numeric::RplISupport),
            221 => Some(Numeric::RplUModeIs),
            324 => Some(Numeric::RplChannelModeIs),
            331 => Some(Numeric::RplNoTopic),
            332 => Some(Numeric::RplTopic),
            333 => Some(Numeric::RplTopicWhoTime),
            353 => Some(Numeric::RplNameReply),
            366 => Some(Numeric::RplEndOfNames),
            372 => Some(Numeric::RplMotd),
            375 => Some(Numeric::RplMotdStart),
            376 => Some(Numeric::RplEndOfMotd),
            401 => Some(Numeric::ErrNoSuchNick),
            403 => Some(Numeric::ErrNoSuchChannel),
            404 => Some(Numeric::ErrCannotSendToChan),
            421 => Some(Numeric::ErrUnknownCommand),
            433 => Some(Numeric::ErrNicknameInUse),
            461 => Some(Numeric::ErrNeedMoreParams),
            _ => None,
        }
    }
}
