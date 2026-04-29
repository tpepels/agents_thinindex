import os
import pathlib as pl
from typing import Optional, TYPE_CHECKING as CHECKING

MAX_RETRIES = 3
local_value = 1


class PromptService:
    DEFAULT_MODEL: str = "base"

    def build_prompt(self):
        LOCAL_CACHE = "skip"
        return os.getenv("PROMPT", self.DEFAULT_MODEL)

    async def fetch_prompt(self):
        return self.build_prompt()


async def create_prompt_service():
    return PromptService()


def helper_function() -> Optional[str]:
    return pl.Path("prompt.txt").name


if TYPE_CHECKING:
    REVEAL_CHECKING = CHECKING
