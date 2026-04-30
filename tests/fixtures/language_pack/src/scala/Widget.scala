package languagepack.scala

import scala.collection.mutable.ListBuffer

trait ScalaRenderable {
  def render(): String
}

enum ScalaMode {
  case Compact
}

class ScalaWidget(name: String) extends ScalaRenderable {
  val ScalaLimit = 4
  var scalaState = name

  def render(
      prefix: String
  ): String = {
    val ignored = "class ScalaStringFake"
    prefix + name
  }
}

object ScalaWidgetFactory {
  type ScalaName = String

  def buildScalaWidget(
      name: String
  ): ScalaWidget = ScalaWidget(name)
}
