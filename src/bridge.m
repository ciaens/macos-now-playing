#import <AppKit/AppKit.h>
#import <MediaPlayer/MediaPlayer.h>

typedef struct {
    const uint8_t *title;
    size_t title_length;
    const uint8_t *artist;
    size_t artist_length;
    const uint8_t *album_title;
    size_t album_title_length;
    double playback_duration;
    double elapsed_playback_time;
    double playback_rate;
    uint64_t playback_queue_index;
    uint64_t playback_queue_count;
    const void *artwork;
} mnp_now_playing_info;

typedef int32_t (*mnp_command_handler)(void *context, double position);
typedef void (*mnp_context_function)(void *context);

static NSString *mnp_string(const uint8_t *bytes, size_t length) {
    if (bytes == NULL) {
        return nil;
    }
    return [[NSString alloc] initWithBytes:bytes length:length encoding:NSUTF8StringEncoding];
}

static MPRemoteCommand *mnp_command(int32_t command) {
    MPRemoteCommandCenter *center = [MPRemoteCommandCenter sharedCommandCenter];
    switch (command) {
        case 0: return center.playCommand;
        case 1: return center.pauseCommand;
        case 2: return center.togglePlayPauseCommand;
        case 3: return center.stopCommand;
        case 4: return center.nextTrackCommand;
        case 5: return center.previousTrackCommand;
        case 6: return center.changePlaybackPositionCommand;
        default: return nil;
    }
}

void mnp_now_playing_set(const mnp_now_playing_info *info) {
    NSMutableDictionary<NSString *, id> *dictionary = [NSMutableDictionary dictionary];
    dictionary[MPMediaItemPropertyTitle] = mnp_string(info->title, info->title_length);
    dictionary[MPMediaItemPropertyArtist] = mnp_string(info->artist, info->artist_length);
    dictionary[MPMediaItemPropertyAlbumTitle] = mnp_string(info->album_title, info->album_title_length);
    dictionary[MPMediaItemPropertyPlaybackDuration] = @(info->playback_duration);
    dictionary[MPNowPlayingInfoPropertyElapsedPlaybackTime] = @(info->elapsed_playback_time);
    dictionary[MPNowPlayingInfoPropertyPlaybackRate] = @(info->playback_rate);
    dictionary[MPNowPlayingInfoPropertyPlaybackQueueIndex] = @(info->playback_queue_index);
    dictionary[MPNowPlayingInfoPropertyPlaybackQueueCount] = @(info->playback_queue_count);
    dictionary[MPNowPlayingInfoPropertyMediaType] = @(MPNowPlayingInfoMediaTypeAudio);
    dictionary[MPMediaItemPropertyArtwork] = (__bridge MPMediaItemArtwork *)info->artwork;
    [MPNowPlayingInfoCenter defaultCenter].nowPlayingInfo = dictionary;
}

void mnp_now_playing_clear(void) {
    [MPNowPlayingInfoCenter defaultCenter].nowPlayingInfo = nil;
}

void mnp_playback_state_set(int32_t state) {
    [MPNowPlayingInfoCenter defaultCenter].playbackState = (MPNowPlayingPlaybackState)state;
}

void *mnp_artwork_new(const uint8_t *bytes, size_t length) {
    NSImage *image = [[NSImage alloc] initWithData:[NSData dataWithBytes:bytes length:length]];
    if (image == nil) {
        return NULL;
    }
    MPMediaItemArtwork *artwork = [[MPMediaItemArtwork alloc] initWithBoundsSize:image.size
                                                                   requestHandler:^NSImage *(__unused CGSize size) {
                                                                       return image;
                                                                   }];
    return (__bridge_retained void *)artwork;
}

void mnp_artwork_free(void *artwork) {
    MPMediaItemArtwork *released = (__bridge_transfer MPMediaItemArtwork *)artwork;
    (void)released;
}

void *mnp_command_add(int32_t command, mnp_command_handler handler, void *context) {
    MPRemoteCommand *remote_command = mnp_command(command);
    if (remote_command == nil) {
        return NULL;
    }
    id target = [remote_command addTargetWithHandler:^MPRemoteCommandHandlerStatus(MPRemoteCommandEvent *event) {
        double position = 0.0;
        if ([event isKindOfClass:[MPChangePlaybackPositionCommandEvent class]]) {
            position = ((MPChangePlaybackPositionCommandEvent *)event).positionTime;
        }
        return (MPRemoteCommandHandlerStatus)handler(context, position);
    }];
    return (__bridge_retained void *)target;
}

void mnp_command_remove(int32_t command, void *target, mnp_context_function free_context, void *context) {
    id handler_target = (__bridge_transfer id)target;
    dispatch_async(dispatch_get_main_queue(), ^{
        [mnp_command(command) removeTarget:handler_target];
        free_context(context);
    });
}

void mnp_run_on_main(mnp_context_function function, void *context) {
    dispatch_async(dispatch_get_main_queue(), ^{
        function(context);
    });
}

void mnp_run_loop_run(double seconds) {
    CFRunLoopRunInMode(kCFRunLoopDefaultMode, seconds, false);
}

void mnp_run_loop_stop_main(void) {
    CFRunLoopStop(CFRunLoopGetMain());
}
