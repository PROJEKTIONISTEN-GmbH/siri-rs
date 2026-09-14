//! Telling other systems how to publish a situation.
//!
//! A situation is not only a description of an event; it also carries instructions
//! about where the event should be announced. Those instructions are *publishing
//! actions*: put this on the web site's home page, run it as a ticker, push it to
//! the mobile app, send it to the pager group of the duty controllers, show it on
//! the platform displays.
//!
//! Two generations of action live side by side in the schema and both are in use.
//! The original set names one action per channel — [`PublishToWebAction`],
//! [`NotifyBySmsAction`] and so on — and says only *whether* to publish there. The
//! later [`PublishingAction`] instead carries the finished passenger text itself, in
//! several languages and several lengths, together with the part of the network it
//! should be shown for; it is what the German VDV 736 profile uses.
//!
//! # Shared shape
//!
//! Every action derives from the same two schema base types, so every action struct
//! in this module starts with the same fields in the same order: `action_status`,
//! then `description`, `action_data` and `publication_window`. The push channels —
//! alerts, SMS, email, pager and direct user notification — add `before_notices` and
//! `clear_notice` after those. The schema expresses this with type extension; Rust
//! has no element-order-preserving inheritance, so the inherited fields are written
//! out on each type, always leading, always in schema order.

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use crate::enumerations::{ActionStatus, Perspective, ScopeType};
use crate::model::OrganisationRef;
use crate::sx::affects::AffectsScope;
use crate::sx::situation::{Image, InfoLink};
use crate::types::{DefaultedText, Duration, Extensions, NaturalLanguageString, ParticipantRef};

siri_ref! {
    /// Identifies one action within the situation that owns it.
    ActionRef;
}

/// Everything that should be done to publish a situation.
///
/// The schema fixes the order of the channel lists, so an `Actions` value written
/// out always groups the web actions before the mobile ones, and so on. Use
/// [`Actions::actions`] to walk them as a single list.
///
/// Actions stated here replace, rather than add to, any actions inherited from the
/// surrounding context.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Actions {
    /// Publish on a web site.
    #[serde(rename = "PublishToWebAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_web_action: Vec<PublishToWebAction>,
    /// Publish in a mobile application.
    #[serde(rename = "PublishToMobileAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_mobile_action: Vec<PublishToMobileAction>,
    /// Publish on a broadcast text service.
    #[serde(rename = "PublishToTvAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_tv_action: Vec<PublishToTvAction>,
    /// Push to subscribers who asked to be alerted.
    #[serde(rename = "PublishToAlertsAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_alerts_action: Vec<PublishToAlertsAction>,
    /// Show on passenger displays at stops or on board.
    #[serde(rename = "PublishToDisplayAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_display_action: Vec<PublishToDisplayAction>,
    /// Something a member of staff has to do by hand.
    #[serde(rename = "ManualAction", default, skip_serializing_if = "Vec::is_empty")]
    pub manual_action: Vec<ManualAction>,
    /// Send by email.
    #[serde(rename = "NotifyByEmailAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_email_action: Vec<NotifyByEmailAction>,
    /// Send by text message.
    #[serde(rename = "NotifyBySmsAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_sms_action: Vec<NotifyBySmsAction>,
    /// Send to a pager.
    #[serde(rename = "NotifyByPagerAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_pager_action: Vec<NotifyByPagerAction>,
    /// Notify a named user or workgroup by some other means.
    #[serde(rename = "NotifyUserAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_user_action: Vec<NotifyUserAction>,
    /// Ready-made passenger information together with the scope it applies to.
    #[serde(rename = "PublishingAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publishing_action: Vec<PublishingAction>,
    /// Implementation-defined content.
    #[serde(rename = "Extensions", default, skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Extensions>,
}

/// One channel action, borrowed from the list it belongs to.
///
/// The schema keeps a separate slot per channel rather than one mixed list, so this
/// enum is the view that puts them back together. Its variants are named for the
/// elements they stand for.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Action<'a> {
    /// Publish on a web site.
    PublishToWebAction(&'a PublishToWebAction),
    /// Publish in a mobile application.
    PublishToMobileAction(&'a PublishToMobileAction),
    /// Publish on a broadcast text service.
    PublishToTvAction(&'a PublishToTvAction),
    /// Push to subscribers who asked to be alerted.
    PublishToAlertsAction(&'a PublishToAlertsAction),
    /// Show on passenger displays.
    PublishToDisplayAction(&'a PublishToDisplayAction),
    /// Something a member of staff has to do by hand.
    ManualAction(&'a ManualAction),
    /// Send by email.
    NotifyByEmailAction(&'a NotifyByEmailAction),
    /// Send by text message.
    NotifyBySmsAction(&'a NotifyBySmsAction),
    /// Send to a pager.
    NotifyByPagerAction(&'a NotifyByPagerAction),
    /// Notify a named user or workgroup by some other means.
    NotifyUserAction(&'a NotifyUserAction),
}

/// Gives a type carrying the schema's channel slots a single-list view of them.
macro_rules! impl_action_list {
    ($ty:ident) => {
        impl $ty {
            /// Every channel action, in the order the schema writes them.
            ///
            /// This flattens the per-channel slots into one list; the ready-made
            /// passenger information of a [`PublishingAction`] is not included,
            /// because it is not a channel instruction.
            pub fn actions(&self) -> Vec<Action<'_>> {
                let mut actions: Vec<Action<'_>> = Vec::new();
                actions.extend(
                    self.publish_to_web_action
                        .iter()
                        .map(Action::PublishToWebAction),
                );
                actions.extend(
                    self.publish_to_mobile_action
                        .iter()
                        .map(Action::PublishToMobileAction),
                );
                actions.extend(
                    self.publish_to_tv_action
                        .iter()
                        .map(Action::PublishToTvAction),
                );
                actions.extend(
                    self.publish_to_alerts_action
                        .iter()
                        .map(Action::PublishToAlertsAction),
                );
                actions.extend(
                    self.publish_to_display_action
                        .iter()
                        .map(Action::PublishToDisplayAction),
                );
                actions.extend(self.manual_action.iter().map(Action::ManualAction));
                actions.extend(
                    self.notify_by_email_action
                        .iter()
                        .map(Action::NotifyByEmailAction),
                );
                actions.extend(
                    self.notify_by_sms_action
                        .iter()
                        .map(Action::NotifyBySmsAction),
                );
                actions.extend(
                    self.notify_by_pager_action
                        .iter()
                        .map(Action::NotifyByPagerAction),
                );
                actions.extend(self.notify_user_action.iter().map(Action::NotifyUserAction));
                actions
            }
        }
    };
}

impl_action_list!(Actions);
impl_action_list!(TextualContent);

/// A name/value pair parameterising an action.
///
/// The schema does not fix what may be configured, so an action's settings that are
/// specific to one operator's publishing system travel as action data rather than as
/// named elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionData {
    /// What the parameter is called.
    #[serde(rename = "Name")]
    pub name: String,
    /// What kind of value the parameter takes.
    #[serde(rename = "Type")]
    pub r#type: String,
    /// The values themselves.
    #[serde(rename = "Value")]
    pub value: Vec<String>,
    /// How to label the parameter when asking a user for it, one entry per language.
    #[serde(rename = "Prompt", default, skip_serializing_if = "Vec::is_empty")]
    pub prompt: Vec<NaturalLanguageString>,
    /// The part of the network the parameter applies to.
    #[serde(rename = "PublishAtScope", default, skip_serializing_if = "Option::is_none")]
    pub publish_at_scope: Option<PublishAtScope>,
}

/// The part of the network an action is to be published for.
///
/// The scope type says at what level the information belongs — a line, a stop place,
/// a whole operator — and `affects` names the actual entities at that level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishAtScope {
    /// The level the information is published at.
    #[serde(rename = "ScopeType")]
    pub scope_type: ScopeType,
    /// The entities at that level.
    #[serde(rename = "Affects")]
    pub affects: AffectsScope,
}

impl PublishAtScope {
    /// A scope naming the given entities at the given level.
    pub fn new(scope_type: ScopeType, affects: AffectsScope) -> Self {
        Self {
            scope_type,
            affects,
        }
    }
}

/// A period with a known start and a known end, both inclusive.
///
/// Publication windows are closed ranges because an information system has to know
/// when to stop showing a message; an open-ended window would leave a stale notice
/// on a display indefinitely.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosedTimestampRange {
    /// The inclusive start of the period.
    #[serde(rename = "StartTime")]
    pub start_time: DateTime<FixedOffset>,
    /// The inclusive end of the period.
    #[serde(rename = "EndTime")]
    pub end_time: DateTime<FixedOffset>,
}

impl ClosedTimestampRange {
    /// A period between two instants, both inclusive.
    pub fn between(start_time: DateTime<FixedOffset>, end_time: DateTime<FixedOffset>) -> Self {
        Self {
            start_time,
            end_time,
        }
    }
}

/// Reminders to send before an action's validity begins.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BeforeNotices {
    /// How long before the start of validity each reminder goes out.
    #[serde(rename = "Interval", default, skip_serializing_if = "Vec::is_empty")]
    pub interval: Vec<Duration>,
}

/// Publish the situation on a web site.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublishToWebAction {
    /// How far the action has got at the moment the situation is published. Absent
    /// means the action is still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be shown. Absent means the windows of the enclosing
    /// level apply.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Include in the site's list of current disruptions. Absent means include.
    #[serde(rename = "Incidents", default, skip_serializing_if = "Option::is_none")]
    pub incidents: Option<bool>,
    /// Include on the site's home page. Absent means do not.
    #[serde(rename = "HomePage", default, skip_serializing_if = "Option::is_none")]
    pub home_page: Option<bool>,
    /// Include in the scrolling news band. Absent means do not.
    #[serde(rename = "Ticker", default, skip_serializing_if = "Option::is_none")]
    pub ticker: Option<bool>,
    /// Social networks to post to, named by host, e.g. `twitter.com`. Anything the
    /// posting needs beyond the name travels as action data.
    #[serde(rename = "SocialNetwork", default, skip_serializing_if = "Vec::is_empty")]
    pub social_network: Vec<String>,
}

/// Publish the situation in a mobile application.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublishToMobileAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be shown.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Include in the app's list of current disruptions. Absent means include.
    #[serde(rename = "Incidents", default, skip_serializing_if = "Option::is_none")]
    pub incidents: Option<bool>,
    /// Include on the app's home screen. Absent means do not.
    #[serde(rename = "HomePage", default, skip_serializing_if = "Option::is_none")]
    pub home_page: Option<bool>,
}

/// Show the situation on passenger displays.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublishToDisplayAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be shown.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Show on displays at stops and stations.
    #[serde(rename = "OnPlace", default, skip_serializing_if = "Option::is_none")]
    pub on_place: Option<bool>,
    /// Show on displays inside vehicles.
    #[serde(rename = "OnBoard", default, skip_serializing_if = "Option::is_none")]
    pub on_board: Option<bool>,
}

/// Push the situation to subscribers who asked to be alerted.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublishToAlertsAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be sent.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Reminders to send before the disruption starts.
    #[serde(rename = "BeforeNotices", default, skip_serializing_if = "Option::is_none")]
    pub before_notices: Option<BeforeNotices>,
    /// Whether to send a further message once the disruption is over.
    #[serde(rename = "ClearNotice", default, skip_serializing_if = "Option::is_none")]
    pub clear_notice: Option<bool>,
    /// Send the alert as email.
    #[serde(rename = "ByEmail", default, skip_serializing_if = "Option::is_none")]
    pub by_email: Option<bool>,
    /// Send the alert to mobile devices.
    #[serde(rename = "ByMobile", default, skip_serializing_if = "Option::is_none")]
    pub by_mobile: Option<bool>,
}

/// Publish the situation on a broadcast text service.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PublishToTvAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be shown.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Publish to the Ceefax service. Absent means publish.
    #[serde(rename = "Ceefax", default, skip_serializing_if = "Option::is_none")]
    pub ceefax: Option<bool>,
    /// Publish to the Teletext service. Absent means publish.
    #[serde(rename = "Teletext", default, skip_serializing_if = "Option::is_none")]
    pub teletext: Option<bool>,
}

/// Something a member of staff has to do by hand to publish the situation.
///
/// The schema adds nothing to the common action fields here: what is to be done is
/// said in `description`, and anything the local process needs travels as action
/// data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ManualAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What is to be done, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for whoever carries the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be published.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
}

/// Send the situation to an individual user by text message.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotifyBySmsAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the message may be sent.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Reminders to send before the disruption starts.
    #[serde(rename = "BeforeNotices", default, skip_serializing_if = "Option::is_none")]
    pub before_notices: Option<BeforeNotices>,
    /// Whether to send a further message once the disruption is over.
    #[serde(rename = "ClearNotice", default, skip_serializing_if = "Option::is_none")]
    pub clear_notice: Option<bool>,
    /// The number to send to, in international form.
    #[serde(rename = "Phone", default, skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    /// Whether the message is charged at a premium rate. Absent means it is not.
    #[serde(rename = "Premium", default, skip_serializing_if = "Option::is_none")]
    pub premium: Option<bool>,
}

/// Send the situation to a workgroup or an individual by email.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotifyByEmailAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the message may be sent.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Reminders to send before the disruption starts.
    #[serde(rename = "BeforeNotices", default, skip_serializing_if = "Option::is_none")]
    pub before_notices: Option<BeforeNotices>,
    /// Whether to send a further message once the disruption is over.
    #[serde(rename = "ClearNotice", default, skip_serializing_if = "Option::is_none")]
    pub clear_notice: Option<bool>,
    /// The address to send to. The schema spells this element in lower case.
    #[serde(rename = "email", default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// Send the situation to a pager.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotifyByPagerAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the message may be sent.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Reminders to send before the disruption starts.
    #[serde(rename = "BeforeNotices", default, skip_serializing_if = "Option::is_none")]
    pub before_notices: Option<BeforeNotices>,
    /// Whether to send a further message once the disruption is over.
    #[serde(rename = "ClearNotice", default, skip_serializing_if = "Option::is_none")]
    pub clear_notice: Option<bool>,
    /// A group of pagers to notify together, e.g. the duty controllers.
    #[serde(rename = "PagerGroupRef", default, skip_serializing_if = "Option::is_none")]
    pub pager_group_ref: Option<String>,
    /// A single pager to notify.
    #[serde(rename = "Pager", default, skip_serializing_if = "Option::is_none")]
    pub pager: Option<String>,
}

/// Notify a named user or workgroup by some means the schema does not name.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NotifyUserAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the notification may be sent.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Reminders to send before the disruption starts.
    #[serde(rename = "BeforeNotices", default, skip_serializing_if = "Option::is_none")]
    pub before_notices: Option<BeforeNotices>,
    /// Whether to send a further message once the disruption is over.
    #[serde(rename = "ClearNotice", default, skip_serializing_if = "Option::is_none")]
    pub clear_notice: Option<bool>,
    /// The workgroup the user belongs to.
    #[serde(rename = "WorkgroupRef", default, skip_serializing_if = "Option::is_none")]
    pub workgroup_ref: Option<String>,
    /// The user's name.
    #[serde(rename = "UserName", default, skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    /// The user's identifier in the sending system.
    #[serde(rename = "UserRef", default, skip_serializing_if = "Option::is_none")]
    pub user_ref: Option<String>,
}

/// Finished passenger information together with the scope it is meant for.
///
/// Where the channel actions say only *where* to publish, this says *what* to
/// publish: the scope names the lines, stops or journeys the text belongs to, and
/// each passenger information entry carries the wording itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PublishingAction {
    /// The part of the network the information is published for.
    #[serde(rename = "PublishAtScope")]
    pub publish_at_scope: PublishAtScope,
    /// The wording, usually one entry per passenger perspective.
    #[serde(rename = "PassengerInformationAction")]
    pub passenger_information_action: Vec<PassengerInformationAction>,
}

impl PublishingAction {
    /// Information to publish for a given scope.
    pub fn new(
        publish_at_scope: PublishAtScope,
        passenger_information_action: Vec<PassengerInformationAction>,
    ) -> Self {
        Self {
            publish_at_scope,
            passenger_information_action,
        }
    }
}

/// One piece of ready-made passenger information.
///
/// It is versioned and timestamped independently of the situation that carries it,
/// so that a wording can be corrected without re-issuing the whole situation, and it
/// is written from one `perspective` — what a passenger waiting at a stop needs to
/// read differs from what a passenger already on the vehicle needs to read.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PassengerInformationAction {
    /// How far the action has got. Absent means still open.
    #[serde(rename = "ActionStatus", default, skip_serializing_if = "Option::is_none")]
    pub action_status: Option<ActionStatus>,
    /// What the action is, in words.
    #[serde(rename = "Description", default, skip_serializing_if = "Option::is_none")]
    pub description: Option<NaturalLanguageString>,
    /// Settings for the publishing system carrying the action out.
    #[serde(rename = "ActionData", default, skip_serializing_if = "Vec::is_empty")]
    pub action_data: Vec<ActionData>,
    /// When the information may be shown. Absent means the windows of the enclosing
    /// level apply.
    #[serde(rename = "PublicationWindow", default, skip_serializing_if = "Vec::is_empty")]
    pub publication_window: Vec<ClosedTimestampRange>,
    /// Identifies this action within the situation.
    #[serde(rename = "ActionRef")]
    pub action_ref: ActionRef,
    /// When the wording was last changed.
    ///
    /// This is the time recorded by the system that originated the text, not by any
    /// hub that passed it on, and it moves with every update.
    #[serde(rename = "RecordedAtTime")]
    pub recorded_at_time: DateTime<FixedOffset>,
    /// The version of this wording, increasing with every update. Absent means it
    /// shares the version of the situation that carries it.
    #[serde(rename = "Version", default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// The system that created the wording. Absent means the situation's own
    /// participant.
    #[serde(rename = "SourceRef", default, skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<ParticipantRef>,
    /// The organisation answerable for the wording.
    #[serde(rename = "OwnerRef", default, skip_serializing_if = "Option::is_none")]
    pub owner_ref: Option<OrganisationRef>,
    /// Whose point of view the wording is written from.
    #[serde(rename = "Perspective", deserialize_with = "crate::xml::token_list::deserialize")]
    pub perspective: Vec<Perspective>,
    /// How important this wording is relative to the situation's others, one being
    /// the most important. Useful for sorting or for dropping the less important
    /// entries on a small display.
    #[serde(rename = "ActionPriority", default, skip_serializing_if = "Option::is_none")]
    pub action_priority: Option<u64>,
    /// The wording itself, usually one entry per size of display.
    #[serde(rename = "TextualContent")]
    pub textual_content: Vec<TextualContent>,
}

/// The passenger wording, cut for one size of display.
///
/// The text is split into parts that answer different questions — what has happened,
/// why, what it means, what to do — so that a receiving system can drop the parts it
/// has no room for instead of truncating a paragraph mid-sentence. Every part
/// repeats per language.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextualContent {
    /// Which size of display the wording is cut for, e.g. `S`, `M` or `L`.
    #[serde(rename = "TextualContentSize", default, skip_serializing_if = "Option::is_none")]
    pub textual_content_size: Option<String>,
    /// Restrict this wording to web publication.
    #[serde(rename = "PublishToWebAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_web_action: Vec<PublishToWebAction>,
    /// Restrict this wording to mobile applications.
    #[serde(rename = "PublishToMobileAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_mobile_action: Vec<PublishToMobileAction>,
    /// Restrict this wording to broadcast text services.
    #[serde(rename = "PublishToTvAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_tv_action: Vec<PublishToTvAction>,
    /// Restrict this wording to alert subscribers.
    #[serde(rename = "PublishToAlertsAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_alerts_action: Vec<PublishToAlertsAction>,
    /// Restrict this wording to passenger displays.
    #[serde(rename = "PublishToDisplayAction", default, skip_serializing_if = "Vec::is_empty")]
    pub publish_to_display_action: Vec<PublishToDisplayAction>,
    /// Restrict this wording to a manual publication process.
    #[serde(rename = "ManualAction", default, skip_serializing_if = "Vec::is_empty")]
    pub manual_action: Vec<ManualAction>,
    /// Restrict this wording to email.
    #[serde(rename = "NotifyByEmailAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_email_action: Vec<NotifyByEmailAction>,
    /// Restrict this wording to text messages.
    #[serde(rename = "NotifyBySmsAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_sms_action: Vec<NotifyBySmsAction>,
    /// Restrict this wording to pagers.
    #[serde(rename = "NotifyByPagerAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_by_pager_action: Vec<NotifyByPagerAction>,
    /// Restrict this wording to direct user notification.
    #[serde(rename = "NotifyUserAction", default, skip_serializing_if = "Vec::is_empty")]
    pub notify_user_action: Vec<NotifyUserAction>,
    /// The headline: what has happened, in one line.
    #[serde(rename = "SummaryContent")]
    pub summary_content: SummaryContent,
    /// Why it has happened.
    #[serde(rename = "ReasonContent", default, skip_serializing_if = "Option::is_none")]
    pub reason_content: Option<ReasonContent>,
    /// Further detail, not repeating the headline.
    #[serde(rename = "DescriptionContent", default, skip_serializing_if = "Vec::is_empty")]
    pub description_content: Vec<DescriptionContent>,
    /// What it means for the passenger's journey.
    #[serde(rename = "ConsequenceContent", default, skip_serializing_if = "Vec::is_empty")]
    pub consequence_content: Vec<ConsequenceContent>,
    /// What the passenger should do instead.
    #[serde(rename = "RecommendationContent", default, skip_serializing_if = "Vec::is_empty")]
    pub recommendation_content: Vec<RecommendationContent>,
    /// How long it is expected to last.
    #[serde(rename = "DurationContent", default, skip_serializing_if = "Option::is_none")]
    pub duration_content: Option<DurationContent>,
    /// Anything else worth saying, e.g. a number to call.
    #[serde(rename = "RemarkContent", default, skip_serializing_if = "Vec::is_empty")]
    pub remark_content: Vec<RemarkContent>,
    /// Links to further information.
    #[serde(rename = "InfoLink", default, skip_serializing_if = "Vec::is_empty")]
    pub info_link: Vec<InfoLink>,
    /// Pictures to show with the wording, e.g. a map of the diversion.
    #[serde(rename = "Image", default, skip_serializing_if = "Vec::is_empty")]
    pub image: Vec<Image>,
    /// Notes for staff only, not to be shown to passengers.
    #[serde(rename = "Internal", default, skip_serializing_if = "Vec::is_empty")]
    pub internal: Vec<DefaultedText>,
}

/// The headline of a piece of passenger information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryContent {
    /// The headline, one entry per language.
    #[serde(rename = "SummaryText")]
    pub summary_text: Vec<DefaultedText>,
}

impl SummaryContent {
    /// A headline in the given languages.
    pub fn new(summary_text: Vec<DefaultedText>) -> Self {
        Self { summary_text }
    }
}

/// Why the disruption has happened, in the passenger's words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReasonContent {
    /// The reason, one entry per language.
    #[serde(rename = "ReasonText")]
    pub reason_text: Vec<DefaultedText>,
}

/// Further detail about the disruption, beyond the headline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DescriptionContent {
    /// The detail, one entry per language.
    #[serde(rename = "DescriptionText")]
    pub description_text: Vec<DefaultedText>,
    /// How important this detail is relative to the others, one being the most
    /// important.
    #[serde(rename = "DescriptionPriority", default, skip_serializing_if = "Option::is_none")]
    pub description_priority: Option<u64>,
}

/// What the disruption means for the passenger's journey.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsequenceContent {
    /// The consequence, one entry per language.
    #[serde(rename = "ConsequenceText")]
    pub consequence_text: Vec<DefaultedText>,
    /// How important this consequence is relative to the others, one being the most
    /// important.
    #[serde(rename = "ConsequencePriority", default, skip_serializing_if = "Option::is_none")]
    pub consequence_priority: Option<u64>,
}

/// What the passenger should do instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecommendationContent {
    /// The recommendation, one entry per language.
    #[serde(rename = "RecommendationText")]
    pub recommendation_text: Vec<DefaultedText>,
    /// How important this recommendation is relative to the others, one being the
    /// most important.
    #[serde(rename = "RecommendationPriority", default, skip_serializing_if = "Option::is_none")]
    pub recommendation_priority: Option<u64>,
}

/// How long the disruption is expected to last.
///
/// An estimate belongs here even when it is uncertain: "until further notice" tells
/// a passenger less than a rough figure they can plan around.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationContent {
    /// The expected duration, one entry per language.
    #[serde(rename = "DurationText")]
    pub duration_text: Vec<DefaultedText>,
}

/// Anything else worth telling the passenger.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemarkContent {
    /// The remark, one entry per language.
    #[serde(rename = "Remark")]
    pub remark: Vec<DefaultedText>,
    /// How important this remark is relative to the others, one being the most
    /// important.
    #[serde(rename = "RemarkPriority", default, skip_serializing_if = "Option::is_none")]
    pub remark_priority: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use quick_xml::de::from_str;
    use quick_xml::se::to_string_with_root;

    #[test]
    fn channel_actions_keep_their_schema_slots_and_read_back_as_one_list() {
        let xml = concat!(
            "<PublishingActions>",
            "<PublishToWebAction><Incidents>true</Incidents><HomePage>true</HomePage>",
            "<Ticker>false</Ticker></PublishToWebAction>",
            "<PublishToWebAction><SocialNetwork>twitter.com</SocialNetwork></PublishToWebAction>",
            "<PublishToMobileAction><Incidents>true</Incidents></PublishToMobileAction>",
            "<PublishToAlertsAction><ByEmail>true</ByEmail><ByMobile>true</ByMobile>",
            "<ClearNotice>true</ClearNotice></PublishToAlertsAction>",
            "</PublishingActions>",
        );

        let actions: Actions = from_str(xml).unwrap();
        assert_eq!(actions.publish_to_web_action.len(), 2);
        assert_eq!(actions.publish_to_mobile_action.len(), 1);
        assert_eq!(
            actions.publish_to_web_action[1].social_network,
            ["twitter.com"]
        );
        assert_eq!(
            actions.publish_to_alerts_action[0].clear_notice,
            Some(true)
        );

        let listed = actions.actions();
        assert_eq!(listed.len(), 4);
        assert!(matches!(listed[0], Action::PublishToWebAction(_)));
        assert!(matches!(listed[2], Action::PublishToMobileAction(_)));
        assert!(matches!(listed[3], Action::PublishToAlertsAction(_)));

        let written = to_string_with_root("PublishingActions", &actions).unwrap();
        let at = |tag: &str| written.find(tag).unwrap_or_else(|| panic!("no {tag}"));
        assert!(at("<PublishToWebAction>") < at("<PublishToMobileAction>"));
        assert!(at("<PublishToMobileAction>") < at("<PublishToAlertsAction>"));
        assert_eq!(from_str::<Actions>(&written).unwrap(), actions);
    }

    /// A passenger information action carrying the inherited fields ahead of its own,
    /// shaped like the German profile's messages.
    const PASSENGER_INFORMATION: &str = concat!(
        "<PassengerInformationAction>",
        "<ActionStatus>open</ActionStatus>",
        "<PublicationWindow><StartTime>2017-05-28T12:47:00+02:00</StartTime>",
        "<EndTime>2017-05-28T13:15:00+02:00</EndTime></PublicationWindow>",
        "<ActionRef>0510</ActionRef>",
        "<RecordedAtTime>2017-05-28T12:47:00+02:00</RecordedAtTime>",
        "<Version>1</Version>",
        "<SourceRef>vbl</SourceRef>",
        "<OwnerRef>VBL</OwnerRef>",
        "<Perspective>general</Perspective>",
        "<TextualContent>",
        "<TextualContentSize>L</TextualContentSize>",
        "<SummaryContent><SummaryText xml:lang=\"DE\">Unterbruch</SummaryText>",
        "<SummaryText xml:lang=\"EN\">Line interruption</SummaryText></SummaryContent>",
        "<RemarkContent><Remark xml:lang=\"EN\">Buses run every ten minutes</Remark>",
        "<RemarkPriority>1</RemarkPriority></RemarkContent>",
        "</TextualContent>",
        "</PassengerInformationAction>",
    );

    #[test]
    fn inherited_action_fields_are_written_before_the_action_s_own() {
        let action: PassengerInformationAction = from_str(PASSENGER_INFORMATION).unwrap();
        assert_eq!(action.action_ref.as_str(), "0510");
        assert_eq!(action.version, Some(1));
        assert_eq!(action.owner_ref.as_ref().unwrap().as_str(), "VBL");
        assert_eq!(action.publication_window.len(), 1);
        assert_eq!(action.perspective.len(), 1);

        let written = to_string_with_root("PassengerInformationAction", &action).unwrap();
        let at = |tag: &str| written.find(tag).unwrap_or_else(|| panic!("no {tag}"));
        assert!(at("<ActionStatus>") < at("<PublicationWindow>"));
        assert!(at("<PublicationWindow>") < at("<ActionRef>"));
        assert!(at("<ActionRef>") < at("<RecordedAtTime>"));
        assert!(at("<Perspective>") < at("<TextualContent>"));
        assert_eq!(
            from_str::<PassengerInformationAction>(&written).unwrap(),
            action
        );
    }

    #[test]
    fn passenger_text_keeps_one_entry_per_language_and_per_content_part() {
        let action: PassengerInformationAction = from_str(PASSENGER_INFORMATION).unwrap();
        let content = &action.textual_content[0];

        assert_eq!(content.textual_content_size.as_deref(), Some("L"));
        assert_eq!(content.summary_content.summary_text.len(), 2);
        assert_eq!(content.summary_content.summary_text[0].value, "Unterbruch");
        assert_eq!(
            content.summary_content.summary_text[1].value,
            "Line interruption"
        );
        assert!(content.reason_content.is_none());
        assert_eq!(content.remark_content[0].remark_priority, Some(1));
        assert!(content.actions().is_empty(), "no channel restriction given");

        let written = to_string_with_root("TextualContent", content).unwrap();
        let at = |tag: &str| written.find(tag).unwrap_or_else(|| panic!("no {tag}"));
        assert!(at("<TextualContentSize>") < at("<SummaryContent>"));
        assert!(at("<SummaryContent>") < at("<RemarkContent>"));
        assert!(!written.contains("ReasonContent"), "absent parts stay absent");
        assert_eq!(from_str::<TextualContent>(&written).unwrap(), *content);

        let mut tagged = content.clone();
        tagged.summary_content.summary_text[0] = DefaultedText::with_lang("DE", "Unterbruch");
        let out = to_string_with_root("TextualContent", &tagged).unwrap();
        assert!(out.contains("<SummaryText xml:lang=\"DE\">Unterbruch</SummaryText>"));
    }

    #[test]
    fn the_email_action_keeps_the_schema_s_lower_case_element_name() {
        let xml = concat!(
            "<NotifyByEmailAction>",
            "<ActionStatus>open</ActionStatus>",
            "<BeforeNotices><Interval>PT30M</Interval><Interval>PT5M</Interval></BeforeNotices>",
            "<ClearNotice>true</ClearNotice>",
            "<email>control@example.org</email>",
            "</NotifyByEmailAction>",
        );

        let action: NotifyByEmailAction = from_str(xml).unwrap();
        assert_eq!(action.email.as_deref(), Some("control@example.org"));
        assert!(action.action_status.is_some());
        assert_eq!(action.before_notices.as_ref().unwrap().interval.len(), 2);

        let written = to_string_with_root("NotifyByEmailAction", &action).unwrap();
        assert!(written.contains("<email>control@example.org</email>"));
        assert!(!written.contains("<Email>"));
        assert_eq!(from_str::<NotifyByEmailAction>(&written).unwrap(), action);
    }

    #[test]
    fn action_data_carries_several_values_and_a_prompt_per_language() {
        let xml = concat!(
            "<ActionData>",
            "<Name>channel</Name>",
            "<Type>string</Type>",
            "<Value>ticker</Value>",
            "<Value>homepage</Value>",
            "<Prompt xml:lang=\"EN\">Which channel?</Prompt>",
            "</ActionData>",
        );

        let data: ActionData = from_str(xml).unwrap();
        assert_eq!(data.name, "channel");
        assert_eq!(data.r#type, "string");
        assert_eq!(data.value, ["ticker", "homepage"]);
        assert_eq!(data.prompt.len(), 1);
        assert!(data.publish_at_scope.is_none());

        let written = to_string_with_root("ActionData", &data).unwrap();
        assert_eq!(written.matches("<Value>").count(), 2);
        assert_eq!(from_str::<ActionData>(&written).unwrap(), data);
    }
}
