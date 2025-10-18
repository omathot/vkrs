use super::Application;

impl Application {
	// fn record_command_buff(&self, img_idx: u32) {
	// 	let cmd_buff = self.command_buff.unwrap();
	// 	let device = self.device.as_ref().unwrap();
	// 	let extent = self.swapchain_extent.unwrap();

	// 	unsafe {
	// 		device
	// 			.begin_command_buffer(cmd_buff, &vk::CommandBufferBeginInfo::default())
	// 			.expect("Should have been able to begin command_buffer")
	// 	};
	// 	// before starting to render, transfer swapchain image to COLOR_ATTACHMENT_OPTIMAL
	// 	self.transition_img_layout(
	// 		img_idx,
	// 		vk::ImageLayout::UNDEFINED,
	// 		vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
	// 		vk::AccessFlags2::empty(), // src access mask (no need to wait for previous op)
	// 		vk::AccessFlags2::COLOR_ATTACHMENT_WRITE, // dst access mask
	// 		vk::PipelineStageFlags2::TOP_OF_PIPE, // src stage
	// 		vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT, // dst stage
	// 	);
	// 	// vk::ClearValue is a union expression, can only hold one field
	// 	let clear_color = vk::ClearValue {
	// 		color: vk::ClearColorValue {
	// 			float32: [0., 0., 0., 1.],
	// 		},
	// 	};
	// 	let attachment_info = vk::RenderingAttachmentInfo {
	// 		image_view: *self
	// 			.swapchain_img_views
	// 			.as_ref()
	// 			.unwrap()
	// 			.get(img_idx as usize)
	// 			.expect("img_idx should always be valid for swapchain img views"),
	// 		image_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
	// 		load_op: vk::AttachmentLoadOp::CLEAR,
	// 		store_op: vk::AttachmentStoreOp::STORE,
	// 		clear_value: clear_color,
	// 		..Default::default()
	// 	};
	// 	let render_info = vk::RenderingInfo {
	// 		render_area: vk::Rect2D {
	// 			offset: vk::Offset2D { x: 0, y: 0 },
	// 			extent: extent,
	// 		},
	// 		layer_count: 1,
	// 		color_attachment_count: 1,
	// 		p_color_attachments: &attachment_info,
	// 		..Default::default()
	// 	};

	// 	unsafe {
	// 		// begin rendering
	// 		device.cmd_begin_rendering(cmd_buff, &render_info);
	// 		// bind pipeline
	// 		device.cmd_bind_pipeline(
	// 			cmd_buff,
	// 			vk::PipelineBindPoint::GRAPHICS,
	// 			self.graphics_pipeline.unwrap(),
	// 		);
	// 		// set dynamic states
	// 		device.cmd_set_viewport(
	// 			cmd_buff,
	// 			0,
	// 			&[vk::Viewport {
	// 				x: 0.,
	// 				y: 0.,
	// 				width: extent.width as f32,
	// 				height: extent.height as f32,
	// 				min_depth: 0.,
	// 				max_depth: 1.,
	// 			}],
	// 		);
	// 		device.cmd_set_scissor(
	// 			cmd_buff,
	// 			0,
	// 			&[vk::Rect2D {
	// 				offset: vk::Offset2D { x: 0, y: 0 },
	// 				extent: extent,
	// 			}],
	// 		);

	// 		device.cmd_draw(cmd_buff, 3, 1, 0, 0);
	// 		device.cmd_end_rendering(cmd_buff);
	// 	};
	// 	// transition back to present to screen
	// 	self.transition_img_layout(
	// 		img_idx,
	// 		vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
	// 		vk::ImageLayout::PRESENT_SRC_KHR,
	// 		vk::AccessFlags2::COLOR_ATTACHMENT_WRITE,
	// 		vk::AccessFlags2::empty(),
	// 		vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
	// 		vk::PipelineStageFlags2::BOTTOM_OF_PIPE,
	// 	);
	// 	unsafe {
	// 		device
	// 			.end_command_buffer(cmd_buff)
	// 			.expect("Should have been able to end cmd buff")
	// 	};
	// }
}
