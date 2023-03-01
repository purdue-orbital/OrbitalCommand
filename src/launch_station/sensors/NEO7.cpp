#include "NEO7.h"
#include <libgpsmm>

NEO7::NEO7(char* host = "localhost", char* port = 2947){
    open(host, port)
}

gps_fix NEO7::read(){
    gpsd_data = read()
    gps_fix fix;

    if (!gpsd_data.status)
    {
        // Raise error No Fix
    }
    
    fix.latitude = gpsd_data.fix.latitude;
    fix.longitude = gpsd_data.fix.longitude;
    fix.alt_agl = gpsd_data.fix.altHAE;
    fix.alt_msl = gpsd_data.fix.altMSL;
    fix.groundspeed = gpsd_data.fix.speed;
    fix.vert_speed = gpsd_data.fix.climb;
    fix.alt_error = gpsd_data.fix.epv;
    fix.pos_error = gps_data.fix.eph;

    return fix;
}