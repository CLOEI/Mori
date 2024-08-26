/*
1.  `oLock the World``|Open inventory and place a `2My First World Lock``.|1|interface/tutorial/tut_npc.rttex|Open inventory and place a `2My First World Lock``.|1
2.  `oBreak Dirt Blocks``|Select the `2Fist`` and break some `2Dirt``!|2|interface/tutorial/tut_npc.rttex|Select the `2Fist`` and break some `2Dirt``!|1
3.  `oCollect Dirt Seeds``|Break the `2Dirt`` to collect `2Dirt Seeds``.|3|interface/tutorial/tut_npc.rttex|Break the `2Dirt`` to collect `2Dirt Seeds``.|1
4.
5.
6.
 */
use std::sync::Arc;
use std::thread;
use crate::bot;
use crate::bot::Bot;
use crate::types::epacket_type::EPacketType;

pub fn lock_the_world(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    thread::spawn(move || {
        bot::send_packet(&bot_clone, EPacketType::NetMessageGenericText, "ftue_start_popup_close`".to_string());
        thread::sleep(std::time::Duration::from_millis(1000));
        bot::place(&bot_clone, 0, -1, 9640);
    });
}

pub fn break_dirt_block(bot: &Arc<Bot>) {
    let bot_clone = bot.clone();
    thread::spawn(move || {
    });
}