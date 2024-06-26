import dayjs from "dayjs";

import duration from "dayjs/plugin/duration";
import isSameOrAfter from "dayjs/plugin/isSameOrAfter";
import minMax from "dayjs/plugin/minMax";
import localizedFormat from "dayjs/plugin/localizedFormat";
import utc from "dayjs/plugin/utc";
import timezone from "dayjs/plugin/timezone";
import relativeTime from "dayjs/plugin/relativeTime";

dayjs.extend(duration);
dayjs.extend(isSameOrAfter);
dayjs.extend(minMax);
dayjs.extend(localizedFormat);
dayjs.extend(utc);
dayjs.extend(timezone);
dayjs.extend(relativeTime);

export default dayjs;
