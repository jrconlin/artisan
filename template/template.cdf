<CHANNEL
    BASE="https://blog.unitedheroes.net/cdf"
    HREF="https://blog.unitedheroes.net/cdf"
    LASTMOD="{{ mod_time }}"
    PRECACHE="NO">
    <TITLE>jr conlin's ink stained banana</TITLE>
    <ABSTRACT>Old school monkey crap. With typewriters.</ABSTRACT>
    <ITEM HREF="https://blog.unitedheroes.net/cdf">
        <USAGE VALUE="ScreenSaver"></USAGE>
    </ITEM>
    {% for post in posts %}
    <ITEM HREF="{{ post.link }}" PRECACHE="No" LASTMOD="{{ post.date }}">
    <TITLE>{{ post.title }}</TITLE>
    <ABSTRACT>{{post.body}}</ABSTRACT>
    </ITEM>
    {% endfor %}
</CHANNEL>