<CHANNEL
    BASE="{{ cdf_url }}"
    HREF="{{ cdf_url }}"
    LASTMOD="{{ mod_time }}"
    PRECACHE="NO">
    <TITLE>{{ blog.name }}</TITLE>
    <ABSTRACT>{{ blog.description }}</ABSTRACT>
    <ITEM HREF="{{ cdf_url }}">
        <USAGE VALUE="ScreenSaver"></USAGE>
    </ITEM>
    {% for post in posts %}
    <ITEM HREF="{{ post.link }}" PRECACHE="No" LASTMOD="{{ post.date }}">
    <TITLE>{{ post.title }}</TITLE>
    <ABSTRACT>{{post.body}}</ABSTRACT>
    </ITEM>
    {% endfor %}
</CHANNEL>