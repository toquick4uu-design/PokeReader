#pragma once

void initialize();
void run_frame();

// value will only ever be modified during run_frame
extern bool PAUSE_REQUESTED;

// Signals rust code to do any relevant clean-up associated with the end of a requested pause
void clear_requested_pause();