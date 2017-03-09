

require 'pathname'
require 'pp'


class String
  def to_name
    self.downcase
  end
end


class Env
  def initialize()
    @tables = [
      @num2ev = {},
      @num2key = {},
      @num2btn = {},
      @num2code = {},
      @num2rel = {},
      @num2abs = {}
    ]
  end

  def define(name, num_expr)
    num = self.instance_eval(num_expr)
    self.class.const_set(name, num)

    insert_table('KEY|BTN', @num2code, name, num)
    insert_table('KEY', @num2key, name, num)
    insert_table('EV', @num2ev, name, num)
    insert_table('BTN', @num2btn, name, num)
    insert_table('REL', @num2rel, name, num)
    insert_table('ABS', @num2abs, name, num)
  end

  def insert_table (prefix, table, name, num)
    if m = name.match(/(?:#{prefix})_(\w+)/)
      table[num] = m[1].to_name # unless table[num]
    end
  end

  def print
    puts('use std::collections::HashMap;')
    puts
    print_table('code', @num2code)
    puts
    print_table('ev', @num2ev)
    puts
    print_table('rel', @num2rel)
    puts
    print_table('abs', @num2abs)
    puts
    print_max_width
  end

  def print_table (name, table)
    puts("pub fn generate_#{name}_name_table() -> HashMap<u16, String> {")
    puts('    let mut table = HashMap::new();')
    table.each {|num, key| puts(%Q[    table.insert(#{num}, "#{key}".to_owned());])}
    puts('    table')
    puts('}')
  end

  def print_max_width
    max = @tables.map {|table| table.max_by {|it| it.last.size } .last.size } .max
    puts("pub static MAX_NAME_SIZE: usize = #{max};")
  end
end


class App
  def initialize
    @env = Env.new
  end

  def read(filepath)
    return false unless filepath.exist?

    File.readlines(filepath).each do |line|
      case line
      when /^#define/
        terms = line.sub(%r[/\*.*], '').split(/\s+/, 3)
        @env.define(terms[1], terms[2]) if terms.size >= 3 and !terms.any?(&:empty?)
      end
    end
  end

  def print
    @env.print
  end
end


if __FILE__ == $0
  app = App.new
  app.read(Pathname('/usr/include/linux/input-event-codes.h')) || app.read(Pathname('/usr/include/linux/input.h'))
  app.print
end
