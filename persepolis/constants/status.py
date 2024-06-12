from enum import Enum


# shutdown_notification
# 0 >> persepolis is running
# 1 >> persepolis is ready for closing(closeEvent  is called)
# 2 >> OK, let's close application!
class ShutdownNotification(Enum):
    Running = 0
    ReadyForClose = 1
    Ok = 2


# checking_flag
# 0 >> normal situation
# 1 >> remove button or delete button pressed or sorting form viewMenu or ... toggled by user
# 2 >> check_download_info function is stopping until remove operation done
# 3 >> deleteFileAction is done it's job and It is called removeButtonPressed.
class CheckingFlag(Enum):
    Normal = 0

    # if checking_flag is equal to 1, it means that user pressed
    # remove or delete button . so checking download information
    # must stop until removing is done! It avoids possibility of crashing!
    RemoveButtonPressed = 1

    StoppingJobs = 2

    DeleteFileActionDone = 3


class DownloadStatus(str, Enum):
    Downloading = 'downloading'
    Waiting = 'waiting'
    Error = 'error'
    Scheduled = 'scheduled'
    Paused = 'paused'
    Stopped = 'stopped'
    Complete = 'complete'

    def __str__(self) -> str:
        return str(self.value)
