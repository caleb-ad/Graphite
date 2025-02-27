use crate::consts::SELECTION_TOLERANCE;
use crate::document::DocumentMessageHandler;
use crate::frontend::utility_types::MouseCursorIcon;
use crate::input::keyboard::MouseMotion;
use crate::input::InputPreprocessorMessageHandler;
use crate::layout::widgets::PropertyHolder;
use crate::message_prelude::*;
use crate::misc::{HintData, HintGroup, HintInfo};
use crate::viewport_tools::tool::{DocumentToolData, Fsm, ToolActionHandlerData};

use graphene::intersection::Quad;
use graphene::Operation;

use glam::DVec2;
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct FillTool {
	fsm_state: FillToolFsmState,
	data: FillToolData,
}

#[remain::sorted]
#[impl_message(Message, ToolMessage, Fill)]
#[derive(PartialEq, Clone, Debug, Hash, Serialize, Deserialize)]
pub enum FillToolMessage {
	// Standard messages
	#[remain::unsorted]
	Abort,

	// Tool-specific messages
	LeftMouseDown,
	RightMouseDown,
}

impl PropertyHolder for FillTool {}

impl<'a> MessageHandler<ToolMessage, ToolActionHandlerData<'a>> for FillTool {
	fn process_action(&mut self, action: ToolMessage, data: ToolActionHandlerData<'a>, responses: &mut VecDeque<Message>) {
		if action == ToolMessage::UpdateHints {
			self.fsm_state.update_hints(responses);
			return;
		}

		if action == ToolMessage::UpdateCursor {
			self.fsm_state.update_cursor(responses);
			return;
		}

		let new_state = self.fsm_state.transition(action, data.0, data.1, &mut self.data, &(), data.2, responses);

		if self.fsm_state != new_state {
			self.fsm_state = new_state;
			self.fsm_state.update_hints(responses);
			self.fsm_state.update_cursor(responses);
		}
	}

	advertise_actions!(FillToolMessageDiscriminant; LeftMouseDown, RightMouseDown);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FillToolFsmState {
	Ready,
}

impl Default for FillToolFsmState {
	fn default() -> Self {
		FillToolFsmState::Ready
	}
}

#[derive(Clone, Debug, Default)]
struct FillToolData {}

impl Fsm for FillToolFsmState {
	type ToolData = FillToolData;
	type ToolOptions = ();

	fn transition(
		self,
		event: ToolMessage,
		document: &DocumentMessageHandler,
		tool_data: &DocumentToolData,
		_data: &mut Self::ToolData,
		_tool_options: &Self::ToolOptions,
		input: &InputPreprocessorMessageHandler,
		responses: &mut VecDeque<Message>,
	) -> Self {
		use FillToolFsmState::*;
		use FillToolMessage::*;

		if let ToolMessage::Fill(event) = event {
			match (self, event) {
				(Ready, lmb_or_rmb) if lmb_or_rmb == LeftMouseDown || lmb_or_rmb == RightMouseDown => {
					let mouse_pos = input.mouse.position;
					let tolerance = DVec2::splat(SELECTION_TOLERANCE);
					let quad = Quad::from_box([mouse_pos - tolerance, mouse_pos + tolerance]);

					if let Some(path) = document.graphene_document.intersects_quad_root(quad).last() {
						let color = match lmb_or_rmb {
							LeftMouseDown => tool_data.primary_color,
							RightMouseDown => tool_data.secondary_color,
							Abort => unreachable!(),
						};
						responses.push_back(DocumentMessage::StartTransaction.into());
						responses.push_back(Operation::SetLayerFill { path: path.to_vec(), color }.into());
						responses.push_back(DocumentMessage::CommitTransaction.into());
					}

					Ready
				}
				_ => self,
			}
		} else {
			self
		}
	}

	fn update_hints(&self, responses: &mut VecDeque<Message>) {
		let hint_data = match self {
			FillToolFsmState::Ready => HintData(vec![HintGroup(vec![
				HintInfo {
					key_groups: vec![],
					mouse: Some(MouseMotion::Lmb),
					label: String::from("Fill with Primary"),
					plus: false,
				},
				HintInfo {
					key_groups: vec![],
					mouse: Some(MouseMotion::Rmb),
					label: String::from("Fill with Secondary"),
					plus: false,
				},
			])]),
		};

		responses.push_back(FrontendMessage::UpdateInputHints { hint_data }.into());
	}

	fn update_cursor(&self, responses: &mut VecDeque<Message>) {
		responses.push_back(FrontendMessage::UpdateMouseCursor { cursor: MouseCursorIcon::Default }.into());
	}
}
