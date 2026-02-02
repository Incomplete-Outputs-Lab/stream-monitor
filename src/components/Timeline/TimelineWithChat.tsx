import React from 'react';
import { StreamTimelineData } from '../../types';
import TimelineChart from './TimelineChart';
import ChatTimelinePanel from './ChatTimelinePanel';

interface TimelineWithChatProps {
  timelineData: StreamTimelineData;
  streamId: number;
  channelId?: number;
}

const TimelineWithChat: React.FC<TimelineWithChatProps> = ({
  timelineData,
  streamId,
  channelId,
}) => {
  return (
    <div className="space-y-6">
      {/* Layout: Timeline on left (70%), Chat panel on right (30%) */}
      <div className="flex flex-col lg:flex-row gap-6">
        {/* Timeline Chart - 70% width on large screens */}
        <div className="flex-1 lg:w-[70%]">
          <TimelineChart timelineData={timelineData} />
        </div>

        {/* Chat Panel - 30% width on large screens */}
        <div className="lg:w-[30%] min-h-[600px]">
          <ChatTimelinePanel streamId={streamId} channelId={channelId} />
        </div>
      </div>
    </div>
  );
};

export default TimelineWithChat;
