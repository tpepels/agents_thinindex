require "json"

RUBY_LIMIT = 4

module RubyPack
  class RubyWidget
    def render
      ignored = "class RubyStringFake; end"
      ignored
    end

    def self.build_ruby_widget(
      name
    )
      new
    end
  end
end
