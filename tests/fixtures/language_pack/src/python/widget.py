from services.core import PythonDependency

PYTHON_LIMIT = 4


class PythonWidget:
    """class PythonStringFake: pass"""

    def render(self):
        return PythonDependency()


def build_python_widget(
    name,
):
    # def PythonCommentFake(): pass
    return PythonWidget()
